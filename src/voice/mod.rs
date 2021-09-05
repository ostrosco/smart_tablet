use crate::settings::{Language, SETTINGS};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam::channel::{unbounded, Receiver};
use std::{
    sync::{Arc, Mutex},
    thread,
};
use webrtc_vad::Vad;

mod command;

// Wrap up the Deepspeech stream so we can send it to our thread.
struct Model(deepspeech::Model);
unsafe impl Send for Model {}

pub fn listen() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the microphone for listening.
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("no output device detected")?;
    let (tx, rx) = unbounded();

    // Once we know we have a microphone, load up the Deepspeech models. We need the expected
    // sample rate for our model so we can confirm that the default microphone in this system
    // can support it.
    let model_path;
    let scorer_path;
    let language;
    {
        let settings = SETTINGS.read().unwrap();
        model_path = settings.voice_settings.model_path.clone();
        scorer_path = settings.voice_settings.scorer_path.clone();
        language = settings.language;
    }
    let mut model = deepspeech::Model::load_from_files(&model_path)?;
    model.enable_external_scorer(&scorer_path)?;
    let expected_sample_rate = model.get_sample_rate() as u32;

    // The Rust bindings of Deepspeech only support mono i16 samples according to the
    // documentation for the model.
    let mut supported_configs = device.supported_input_configs()?;
    let config = supported_configs
        .find(|c| c.channels() == 1 && c.sample_format() == cpal::SampleFormat::I16)
        .ok_or("no supported format for microphone")?
        .with_sample_rate(cpal::SampleRate(expected_sample_rate))
        .config();
    println!("config: {:?}", config);
    let model = Arc::new(Mutex::new(Model(model)));

    // Start the input stream. In order to avoid issues with latency processing the samples, we
    // have the stream just send the data out across a channel versus doing the processing in the
    // callback itself.
    let input_stream = device.build_input_stream(
        &config,
        move |data: &[i16], _: &cpal::InputCallbackInfo| {
            tx.send(data.to_vec()).expect("Couldn't send audio data");
        },
        move |err| {
            eprintln!("Error collecting data: {}", err);
        },
    )?;
    input_stream.play()?;

    let handle = thread::spawn(move || {
        process_audio(model, rx, language);
    });
    handle.join().unwrap();
    Ok(())
}

/// Receive, process, and transcribe received audio from the microphone.
///
fn process_audio(model: Arc<Mutex<Model>>, rx: Receiver<Vec<i16>>, language: Language) {
    // A constant that keeps track of the number of samples we're going to hold on to.
    const SAMPLE_HISTORY_LEN: u32 = 3;

    // A constant that keeps track of how long of silence do we wait before attempting to
    // transcribe audio with Deepspeech.
    const NUM_SILENT_SAMPLES: u32 = 3;

    let command_parser =
        command::CommandParser::init(language).expect("Can't load the command file");

    // Since this thread owns the model and will be using it exclusively, we'll just lock the mutex
    // at the beginning and don't bother letting go.
    let model = &mut model.lock().unwrap().0;

    let mut stream = None;

    // We know that Deepspeech only works with 16kHz data so we hard code it here.
    let mut vad = Vad::new_with_rate_and_mode(
        webrtc_vad::SampleRate::Rate16kHz,
        webrtc_vad::VadMode::Aggressive,
    );

    let mut silent_count = 0;
    let mut speech_found = false;

    let silence_level = 1000;
    let mut prev_sample = vec![];
    let mut num_samples = 0;

    rx.iter().for_each(move |mut samps| {
        // Since we're dropping the stream after we finish a decode, we need to check each
        // iteration to see if the stream needs to be re-created.
        if stream.is_none() {
            stream = Some(model.create_stream().expect("couldn't create stream"));
        }

        // Since short words seem to be missed here, we look for _any_ detection in our stream to
        // decide whether or not this gets placed in the stream for processing.
        let (min_amplitude, max_amplitude) = samps
            .iter()
            .fold((std::i16::MAX, std::i16::MIN), |acc, samp| {
                (*samp.min(&acc.0), *samp.max(&acc.1))
            });

        // Don't bother trying voice activity detection unless we're over some amplitude.
        if min_amplitude > -silence_level && max_amplitude < silence_level && !speech_found {
            // Though we aren't going any voice activity detection, Deepspeech will very frequently
            // drop the first word in a phrase if there's no silence preceeding it. To that end, we
            // save off up to SAMPLE_HISTORY_LEN number of samples and append when we start picking
            // up something worth trying voice detection on.
            if num_samples == 0 {
                prev_sample = samps.to_vec();
                num_samples += 1;
            } else if num_samples <= SAMPLE_HISTORY_LEN {
                prev_sample.append(&mut samps);
                num_samples += 1;
            } else {
                prev_sample.drain(0..samps.len());
                prev_sample.append(&mut samps);
            }
            if silent_count <= NUM_SILENT_SAMPLES {
                silent_count += 1;
            }
        } else {
            prev_sample.append(&mut samps);
            let is_speech = prev_sample
                .chunks_exact(160)
                .any(|frame| vad.is_voice_segment(frame) == Ok(true));

            if is_speech {
                silent_count = 0;
                speech_found = true;
            } else if silent_count <= NUM_SILENT_SAMPLES {
                silent_count += 1;
            }
        }

        // If speech has been found at all, start dumping data into the stream...even the silence.
        // This results in a much better parse by Deepspeech.
        if speech_found {
            stream.as_mut().unwrap().feed_audio(&prev_sample);
            prev_sample.clear();
            num_samples = 0;
        }

        // The silent count here is a magic number we're using that we found mostly through
        // experimentation. At some point we need to figure out a better way to handle this.
        if silent_count >= NUM_SILENT_SAMPLES && speech_found {
            let mut stream_taken = stream.take().unwrap();

            // Due to the rather shocking false positive rate of webrtc-vad we're running into,
            // We have this as a stopgap. Do an intermediate decode and if Deepspeech says we've
            // got nothing, just continue collecting data into the stream.
            if let Ok(val) = stream_taken.intermediate_decode() {
                if val != String::new() {
                    println!("Decoded text: {:?}", val);
                    println!("Command: {:?}", command_parser.parse(&val));
                    drop(stream_taken);
                    silent_count = 0;
                }
                speech_found = false;
            }
        }
    });
}

use crate::{
    settings::{Language, SETTINGS},
    voice::command::Command,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::thread;
use tokio::{self, sync::mpsc};
use webrtc_vad::Vad;

pub mod command;
mod number;

// Wrap up the Deepspeech stream so we can send it to our thread.
struct Model(deepspeech::Model);
unsafe impl Send for Model {}

pub struct VoiceProcessing {
    command_tx: mpsc::UnboundedSender<Command>,
}

impl VoiceProcessing {
    pub fn new(command_tx: mpsc::UnboundedSender<Command>) -> Self {
        Self { command_tx }
    }

    pub fn start_listeners(&mut self) {
        let (audio_samples_tx, audio_samples_rx) = mpsc::unbounded_channel();

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
            if !std::path::Path::new(&model_path).exists() {
                eprintln!("model path is invalid");
                return;
            }
            if !std::path::Path::new(&scorer_path).exists() {
                eprintln!("scorer path is invalid");
                return;
            }
            language = settings.language;
        }
        let sample_rate;
        {
            let model =
                deepspeech::Model::load_from_files(&model_path).expect("couldn't load models");
            sample_rate = model.get_sample_rate() as u32;
        }

        thread::spawn(move || match listen(audio_samples_tx, sample_rate) {
            Ok(_) => eprintln!("Voice listener terminated without an error, but shouldn't"),
            Err(e) => eprintln!("Voice listener terminated with error: {:?}", e),
        });

        let command_tx = self.command_tx.clone();
        thread::spawn(move || {
            match process_audio(
                &model_path,
                &scorer_path,
                audio_samples_rx,
                command_tx,
                language,
            ) {
                Ok(_) => eprintln!("Voice processor terminated without an error, but shouldn't"),
                Err(e) => eprintln!("Voice processor terminated with error: {:?}", e),
            }
        });
    }
}

fn listen(
    tx: mpsc::UnboundedSender<Vec<i16>>,
    sample_rate: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Configure the microphone for listening.
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("no output device detected")?;

    // The Rust bindings of Deepspeech only support mono i16 samples according to the
    // documentation for the model.
    let mut supported_configs = device.supported_input_configs()?;
    let config = supported_configs
        .find(|c| c.channels() == 1 && c.sample_format() == cpal::SampleFormat::I16)
        .ok_or("no supported format for microphone")?
        .with_sample_rate(cpal::SampleRate(sample_rate))
        .config();
    println!("config: {:?}", config);

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
    loop {}
}

/// Receive, process, and transcribe received audio from the microphone.
///
fn process_audio(
    model_path: &std::path::Path,
    scorer_path: &std::path::Path,
    mut audio_samples_rx: mpsc::UnboundedReceiver<Vec<i16>>,
    command_tx: mpsc::UnboundedSender<Command>,
    language: Language,
) -> Result<(), Box<dyn std::error::Error>> {
    // A constant that keeps track of the number of samples we're going to hold on to.
    const SAMPLE_HISTORY_LEN: u32 = 3;

    // A constant that keeps track of how long of silence do we wait before attempting to
    // transcribe audio with Deepspeech.
    const NUM_SILENT_SAMPLES: u32 = 3;

    let command_parser = command::CommandParser::init(language)?;

    let mut stream = None;
    let mut model = deepspeech::Model::load_from_files(model_path)?;
    model.enable_external_scorer(scorer_path)?;

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

    loop {
        let mut samps = audio_samples_rx
            .blocking_recv()
            .ok_or("audio sample stream has closed")?;

        // Since we're dropping the stream after we finish a decode, we need to check each
        // iteration to see if the stream needs to be re-created.
        if stream.is_none() {
            stream = Some(model.create_stream()?);
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
                    let command = command_parser.parse(&val);
                    println!("Command: {:?}", command);
                    drop(stream_taken);

                    silent_count = 0;
                    if let Some(cmd) = command {
                        command_tx.send(cmd)?;
                    }
                }
                speech_found = false;
            }
        }
    }
}

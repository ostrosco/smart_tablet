use crate::settings::SETTINGS;
use crossbeam::channel::{Receiver, unbounded};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use nnnoiseless::DenoiseState;
use std::{
    ops::Neg,
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

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
    {
        let settings = SETTINGS.read().unwrap();
        model_path = settings.voice_settings.model_path.clone();
        scorer_path = settings.voice_settings.scorer_path.clone();
    }
    let mut model = deepspeech::Model::load_from_files(&model_path)?;
    model.enable_external_scorer(&scorer_path)?;
    let expected_sample_rate = model.get_sample_rate() as u32;

    // The Rust bindings of Deepspeech only support mono i16 samples according to the
    // documentation for the model.
    let mut supported_configs = device.supported_input_configs()?;
    let config = supported_configs
        .find(|c| c.channels() == 1 && c.sample_format() == cpal::SampleFormat::F32)
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
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            tx.send(data.to_vec()).expect("Couldn't send audio data");
        },
        move |err| {
            eprintln!("Error collecting data: {}", err);
        },
    )?;
    input_stream.play()?;

    let handle = thread::spawn(move || {
        process_audio(model, rx, expected_sample_rate);
    });
    handle.join().unwrap();
    Ok(())
}

/// Receive, process, and transcribe received audio from the microphone.
///
fn process_audio(model: Arc<Mutex<Model>>, rx: Receiver<Vec<f32>>, sample_rate: u32) {
    // These are magic numbers for the recording. These vary depending on the background noise, the
    // quality of the microphone, and the cadence in which the user speaks. We'll need to figure
    // out a way to calculate these or at least make them configurable.
    let silence_amplitude: f32 = 0.1;
    let silence_time: f32 = 0.4;

    // Since this thread owns the model and will be using it exclusively, we'll just lock the mutex
    // at the beginning and don't bother letting go.
    let model = &mut model.lock().unwrap().0;

    let mut silence_start: Option<std::time::Instant> = None;
    let mut silence_has_broken = false;

    // We're configuring our VecDeque to hold up to ten seconds of audio. We'll manually drain off
    // samples that exceed this length as we receive samples. This should be enough for most
    // practical purposes.
    let secs_of_audio = 10;
    let queue_length = sample_rate as usize * secs_of_audio;
    let mut audio_until_pause: Vec<f32> = Vec::with_capacity(queue_length);

    rx.iter().for_each(move |mut samps| {
        let (max_amplitude, min_amplitude) =
            samps.iter().fold((std::f32::NAN, std::f32::NAN), |m, v| {
                (m.0.max(*v), m.1.min(*v))
            });
        let is_silent =
            max_amplitude < silence_amplitude && min_amplitude > silence_amplitude.neg();

        if audio_until_pause.len() >= queue_length {
            audio_until_pause.drain(0..samps.len());
        }
        audio_until_pause.append(&mut samps);

        if is_silent && silence_has_broken {
            match silence_start {
                None => silence_start = Some(Instant::now()),
                Some(silent_time) => {
                    if silent_time.elapsed().as_secs_f32() >= silence_time {
                        // Denoise the audio and scale the values to the range of an i16,
                        // converting to i16 at the same time. Deepspeech can only operate on i16
                        // values.
                        let denoised_audio = denoise(&audio_until_pause)
                            .iter()
                            .map(|f| (*f * std::i16::MAX as f32) as i16)
                            .collect::<Vec<i16>>();

                        // For now we just print out the text we receive after de-noising. Later
                        // on we would expect this to actually take the transcribed audio and
                        // process it for commands.
                        println!("{:?}", model.speech_to_text(&denoised_audio));
                        audio_until_pause.clear();
                        silence_has_broken = false;
                    }
                }
            }
        } else if !is_silent {
            silence_start = None;
            silence_has_broken = true;
        }
    });
}

/// A utility function to denoise the recording using nnnoiseless before we transcribe it.
fn denoise(audio: &[f32]) -> Vec<f32> {
    let mut output = Vec::new();
    let mut buf = [0.0; DenoiseState::FRAME_SIZE];
    let mut denoise = DenoiseState::new();
    let mut first = true;
    audio
        .chunks_exact(DenoiseState::FRAME_SIZE)
        .for_each(|chunk| {
            denoise.process_frame(&mut buf[..], chunk);
            if !first {
                output.extend_from_slice(&buf[..])
            }
            first = false;
        });
    output
}

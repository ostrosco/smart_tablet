use crate::settings::SETTINGS;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use deepspeech::Model;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Instant,
};

// Wrap up the Deepspeech stream so we can send it to our thread.
struct ModelStream(deepspeech::Stream);
unsafe impl Send for ModelStream {}

pub fn listen() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the microphone for listening.
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or("no output device detected")?;
    let (tx, rx) = mpsc::channel();

    // Once we know we have a microphone, load up the Deepspeech models. We need the expected
    // sample rate for our model so we can confirm that the default microphone in this system
    // can support it.
    let model_path;
    {
        let settings = SETTINGS.read().unwrap();
        model_path = settings.voice_settings.model_path.clone();
    }
    let mut model = Model::load_from_files(&model_path)?;
    let expected_sample_rate = model.get_sample_rate() as u32;

    // The Rust bindings of Deepspeech only support mono i16 samples according to the
    // documentation for the model.
    let mut supported_configs = device.supported_input_configs()?;
    let config = supported_configs
        .find(|c| c.channels() == 1 && c.sample_format() == cpal::SampleFormat::I16)
        .ok_or("no supported format for microphone")?
        .with_sample_rate(cpal::SampleRate(expected_sample_rate))
        .config();
    let model_stream = Arc::new(Mutex::new(ModelStream(model.create_stream()?)));

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
        decode(model_stream, rx);
    });
    handle.join().unwrap();
    Ok(())
}

/// This is a very dummy decode system and does not represent how any of the decoding would be
/// expected to work officially. This merely exists as a WIP to demonstrate that we can actually
/// process audio samples with Deepspeech.
fn decode(stream: Arc<Mutex<ModelStream>>, rx: mpsc::Receiver<Vec<i16>>) {
    let mut duration = Instant::now();
    rx.iter().for_each(move |samp| {
        let mut stream = stream.lock().unwrap();
        stream.0.feed_audio(&samp[..]);
        if duration.elapsed().as_secs() >= 10 {
            println!("Current text: {:?}", stream.0.intermediate_decode());
            duration = Instant::now();
        }
    });
}

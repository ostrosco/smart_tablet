use crate::{
    news::{rss_news::RssNewsSource, NewsSource},
    weather::{TemperatureUnits, WeatherSource},
};
use actix_web::{http::header::ContentEncoding, web, HttpResponse};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::{Read, Write},
    path::PathBuf,
    sync::RwLock,
};

lazy_static! {
    // The main settings for all user-controlled settings. Settings are stored in a local
    // JSON file for persistance in between runs.
    pub static ref SETTINGS: RwLock<Settings> = {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("settings.json")
            .expect("couldn't create settings file");
        let metadata = file.metadata().expect("couldn't get file metadata");
        if metadata.len() == 0 {
            // If the file is zero sized, we must have just created it. Just use the default
            // settings and write it to the file so that way it's populated with something.
            let settings = Settings::default();
            write!(file, "{}", serde_json::to_string_pretty(&settings).unwrap())
                .expect("couldn't write to settings file");
            RwLock::new(settings)
        } else {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer).expect("couldn't read settings file");
            match serde_json::from_str(&buffer) {
                Ok(settings) => RwLock::new(settings),
                Err(e) => {
                    eprintln!("Unable to load settings file: {:?}, using defaults", e);
                    RwLock::new(Settings::default())
                }
            }
        }
    };
}

// Sorry, I'm only supporting English as it's the only language I know and the Deepspeech models
// are only trained for English. I'm adding the hooks for other language support if sometime in the
// future someone else would like to add support.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Language {
    English,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WeatherSettings {
    pub weather_source: WeatherSource,
    pub temp_units: TemperatureUnits,
    // Time between API queries in minutes.
    pub polling_rate: u32,
    pub api_key: String,
    pub lat: f32,
    pub lon: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewsSettings {
    pub news_sources: HashSet<NewsSource>,
    // Time between API queries in minutes.
    pub polling_rate: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoiceSettings {
    pub model_path: PathBuf,
    pub scorer_path: PathBuf,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub weather_settings: WeatherSettings,
    pub news_settings: NewsSettings,
    pub voice_settings: VoiceSettings,
    pub language: Language,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            weather_settings: WeatherSettings {
                weather_source: WeatherSource::OpenWeather,
                temp_units: TemperatureUnits::Celsius,
                polling_rate: 3600,
                api_key: String::new(),
                lat: 0.0,
                lon: 0.0,
            },
            news_settings: NewsSettings {
                news_sources: [
                    NewsSource::Rss(RssNewsSource::NPR),
                    NewsSource::Rss(RssNewsSource::BBC),
                ]
                .iter()
                .cloned()
                .collect(),
                polling_rate: 3600,
            },
            voice_settings: VoiceSettings {
                model_path: PathBuf::new(),
                scorer_path: PathBuf::new(),
            },
            language: Language::English,
        }
    }
}

// Update the settings. Currently, the client has to send all settings at once; perhaps in the
// future we can allow for individual changes of settings if it makes sense.
pub async fn change_settings(settings: web::Json<Settings>) -> HttpResponse {
    let settings = settings.into_inner();
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("settings.json")
        .expect("couldn't open settings file");
    write!(file, "{}", serde_json::to_string_pretty(&settings).unwrap())
        .expect("couldn't write to settings file");
    *SETTINGS.write().unwrap() = settings;
    HttpResponse::Ok().finish()
}

// Responds with all current settings in JSON format.
pub async fn get_settings() -> HttpResponse {
    let settings = SETTINGS.read().unwrap();
    let settings_resp = serde_json::to_string(&*settings).unwrap();
    HttpResponse::Ok()
        .content_type("application/json")
        .body(settings_resp)
}

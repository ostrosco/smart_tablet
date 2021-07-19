use crate::{service::Service, settings::SETTINGS};
use actix_rt::time::interval;
use async_trait::async_trait;
use chrono::NaiveDate;
use futures::channel::mpsc;
use serde::{Deserialize, Serialize};
use std::time::Duration;

mod openweather;
use openweather::{OpenWeatherCurrent, OpenWeatherForecast, OpenWeatherReport};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WeatherSource {
    OpenWeather,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TemperatureUnits {
    Kelvin,
    Celsius,
    Fahrenheit,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WeatherReport {
    current_weather: CurrentWeather,
    forecast: Vec<Forecast>,
}

impl From<OpenWeatherReport> for WeatherReport {
    fn from(report: OpenWeatherReport) -> Self {
        let current_weather = report.current.into();
        let forecast = report.daily.iter().map(|x| x.into()).collect();
        Self {
            current_weather,
            forecast,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CurrentWeather {
    temp: f32,
    humidity: f32,
    description: String,
}

impl From<OpenWeatherCurrent> for CurrentWeather {
    fn from(current: OpenWeatherCurrent) -> Self {
        Self {
            temp: current.temp,
            humidity: current.humidity,
            description: current.description,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Forecast {
    date: NaiveDate,
    min_temp: f32,
    max_temp: f32,
    humidity: f32,
    rain_chance: f32,
    cloudiness: f32,
    description: String,
}

impl From<&OpenWeatherForecast> for Forecast {
    fn from(forecast: &OpenWeatherForecast) -> Self {
        Self {
            date: forecast.date,
            min_temp: forecast.min_temp,
            max_temp: forecast.max_temp,
            humidity: forecast.humidity,
            rain_chance: forecast.rain_chance * 100.0,
            cloudiness: forecast.cloudiness,
            description: forecast.description.clone(),
        }
    }
}

#[derive(Clone)]
pub struct WeatherService {
    tx: Option<mpsc::Sender<Box<dyn erased_serde::Serialize + Send + Sync>>>,
}

impl WeatherService {
    pub fn new() -> Self {
        Self { tx: None }
    }
}

#[async_trait]
impl Service for WeatherService {
    fn set_sender(&mut self, tx: mpsc::Sender<Box<dyn erased_serde::Serialize + Send + Sync>>) {
        self.tx = Some(tx);
    }

    async fn start_service(&mut self) {
        // The interval between queries of the weather API is set at the start of the application
        // so changing the setting afterwards doesn't have any effect at the moment.
        let polling_rate;
        {
            let settings = SETTINGS.read().unwrap();
            polling_rate = settings.weather_settings.polling_rate as u64;
        }
        let mut interval = interval(Duration::from_secs(polling_rate));
        loop {
            match self.get_weather_report().await {
                Ok(report) => {
                    if let Some(tx) = &mut self.tx {
                        if tx.try_send(Box::new(report)).is_err() {
                            eprintln!("Reciever has been closed.");
                        }
                    } else {
                        eprintln!("News transmitter not set.");
                    }
                }
                Err(e) => eprintln!("Couldn't get weather: {:?}", e),
            }
            interval.tick().await;
        }
    }

    fn get_service_name(&self) -> String {
        WeatherService::get_service_name()
    }
}

impl WeatherService {
    /// Gets the current weather from the user's chosen weather provider. Polls current weather
    /// settings information prior to querying for weather.
    async fn get_weather_report(
        &self,
    ) -> Result<WeatherReport, Box<dyn std::error::Error + Send + Sync>> {
        let weather_source;
        let api_key;
        let temp_units;
        let lat;
        let lon;
        {
            let settings = SETTINGS.read().unwrap();
            let weather_settings = &settings.weather_settings;
            weather_source = weather_settings.weather_source.clone();
            api_key = weather_settings.api_key.clone();
            temp_units = weather_settings.temp_units.clone();
            lon = weather_settings.lon;
            lat = weather_settings.lat;
        }
        match weather_source {
            WeatherSource::OpenWeather => {
                openweather::get_weather(lat, lon, temp_units, api_key).await
            }
        }
    }

    pub fn get_service_name() -> String {
        String::from("Weather")
    }
}

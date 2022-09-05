use crate::{message::UpdateMessage, service::Service, settings::SETTINGS};
use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

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
    polling_rate: u64,
}

impl WeatherService {
    pub fn new() -> Self {
        let polling_rate;
        {
            let settings = SETTINGS.read().unwrap();
            polling_rate = settings.weather_settings.polling_rate as u64;
        }
        Self { polling_rate }
    }

    pub fn get_name() -> String {
        String::from("Weather")
    }
}

#[async_trait]
impl Service for WeatherService {
    fn get_polling_rate(&self) -> u64 {
        self.polling_rate
    }

    fn get_service_name(&self) -> String {
        WeatherService::get_name()
    }

    /// Gets the current weather from the user's chosen weather provider. Polls current weather
    /// settings information prior to querying for weather.
    async fn update(&self) -> UpdateMessage {
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
        let weather_report = match weather_source {
            WeatherSource::OpenWeather => {
                openweather::get_weather(lat, lon, temp_units, api_key).await
            }
        };

        if let Ok(wr) = weather_report {
            UpdateMessage::Weather(wr)
        } else {
            UpdateMessage::Error("unable to get weather information".into())
        }
    }
}

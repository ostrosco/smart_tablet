use crate::settings::SETTINGS;
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

pub struct Weather();

impl Weather {
    pub async fn get_weather_report(
        &self,
        lat: f32,
        lon: f32,
    ) -> Result<WeatherReport, Box<dyn std::error::Error>> {
        let weather_source;
        let api_key;
        let temp_units;
        {
            let settings = SETTINGS.read().unwrap();
            weather_source = settings.weather_settings.weather_source.clone();
            api_key = settings.weather_settings.api_key.clone();
            temp_units = settings.weather_settings.temp_units.clone();
        }
        match weather_source {
            WeatherSource::OpenWeather => {
                openweather::get_weather(lat, lon, temp_units, api_key).await
            }
        }
    }
}

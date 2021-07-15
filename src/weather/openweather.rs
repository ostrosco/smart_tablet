use crate::weather::{TemperatureUnits, WeatherReport};
use chrono::{NaiveDate, NaiveDateTime};
use serde::{de, Deserialize, Deserializer};

/// A structure reprsenting the results from OpenWeather's OneCall API. We only currently
/// support the current weather and the daily forecast; all other data from the OneCall
/// API call is discarded.
#[derive(Deserialize)]
pub struct OpenWeatherReport {
    pub current: OpenWeatherCurrent,
    pub daily: Vec<OpenWeatherForecast>,
}

pub struct OpenWeatherCurrent {
    pub temp: f32,
    pub humidity: f32,
    pub description: String,
}

// Due to the heavily nested nature of the JSON (and the fact that we're really only interested in
// one field out of most of these), we're writing our own deserializer to avoid having to deal with
// nested structures upon nested structures.
impl<'de> Deserialize<'de> for OpenWeatherCurrent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TopLevel {
            temp: f32,
            humidity: f32,
            weather: Vec<Weather>,
        }

        #[derive(Deserialize)]
        struct Weather {
            description: String,
        }

        let helper = TopLevel::deserialize(deserializer)?;
        Ok(Self {
            temp: helper.temp,
            humidity: helper.humidity,
            description: helper.weather[0].description.clone(),
        })
    }
}

pub struct OpenWeatherForecast {
    pub date: NaiveDate,
    pub min_temp: f32,
    pub max_temp: f32,
    pub humidity: f32,
    pub rain_chance: f32,
    pub cloudiness: f32,
    pub description: String,
}

// Due to the heavily nested nature of the JSON (and the fact that we're really only interested in
// one field out of most of these), we're writing our own deserializer to avoid having to deal with
// nested structures upon nested structures.
impl<'de> Deserialize<'de> for OpenWeatherForecast {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TopLevel {
            #[serde(rename = "dt")]
            forecast_time: i64,
            temp: Temperature,
            humidity: f32,
            // For some reason, the OpenWeather API wraps the weather with the description into an
            // array in the JSON. As far as we've seen, this only ever returns one element so for
            // now we're just going to pull out the first value and use it.
            weather: Vec<Weather>,
            clouds: f32,
            #[serde(rename = "pop")]
            rain_chance: f32,
        }

        #[derive(Deserialize)]
        struct Temperature {
            min: f32,
            max: f32,
        }

        #[derive(Deserialize)]
        struct Weather {
            description: String,
        }

        let helper = TopLevel::deserialize(deserializer)?;
        let date = NaiveDateTime::from_timestamp(helper.forecast_time, 0).date();

        // As described earlier, we're only using the first value. This is just a sanity check
        // since there is a possibility that this array we're getting out of the JSON could be
        // empty but I'm not sure if this is _actually_ possible in practice.
        let description = match helper.weather.get(0) {
            Some(weather) => weather.description.clone(),
            None => return Err(de::Error::custom("missing field weather")),
        };
        Ok(Self {
            date,
            min_temp: helper.temp.min,
            max_temp: helper.temp.max,
            humidity: helper.humidity,
            cloudiness: helper.clouds,
            rain_chance: helper.rain_chance,
            description,
        })
    }
}

/// Query weather from OpenWeather's OneCall API.
pub async fn get_weather(
    lat: f32,
    lon: f32,
    temp_units: TemperatureUnits,
    api_key: String,
) -> Result<WeatherReport, Box<dyn std::error::Error + Send + Sync>> {
    let units = match temp_units {
        TemperatureUnits::Kelvin => "standard",
        TemperatureUnits::Celsius => "metric",
        TemperatureUnits::Fahrenheit => "imperial",
    };
    let uri = format!(
        "http://api.openweathermap.org/data/2.5/onecall?lat={}&lon={}&appid={}&units={}",
        lat, lon, api_key, units,
    );
    let resp: OpenWeatherReport = reqwest::get(uri).await?.json().await?;
    Ok(resp.into())
}

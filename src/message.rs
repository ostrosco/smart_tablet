use crate::news;
use crate::weather;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateMessage {
    Weather(weather::WeatherReport),
    News(Vec<news::NewsItem>),
}

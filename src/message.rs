use crate::news;
use crate::weather;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum UpdateMessage {
    Weather(weather::WeatherReport),
    News(Vec<news::NewsItem>),
}

use crate::settings::SETTINGS;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

pub mod rss_news;
use rss_news::RssNewsSource;

#[derive(Serialize, Deserialize)]
pub struct NewsItem {
    source: String,
    title: Option<String>,
    description: Option<String>,
    pub_date: Option<DateTime<FixedOffset>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum NewsSource {
    Rss(RssNewsSource),
}

impl NewsSource {
    pub async fn get_news(&self) -> Result<Vec<NewsItem>, Box<dyn std::error::Error>> {
        match self {
            NewsSource::Rss(source) => source.get_news().await,
        }
    }
}

pub async fn get_news() -> Result<Vec<NewsItem>, Box<dyn std::error::Error>> {
    let news_sources;
    {
        let settings = SETTINGS.read().unwrap();
        news_sources = settings.news_settings.news_sources.clone();
    }
    let mut news_list = Vec::new();
    for source in news_sources {
        match source.get_news().await {
            Ok(mut news) => news_list.append(&mut news),
            Err(_) => continue,
        }
    }
    Ok(news_list)
}

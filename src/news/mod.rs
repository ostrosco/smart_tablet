use crate::message::UpdateMessage;
use crate::{service::Service, settings::SETTINGS};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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

#[derive(Clone)]
pub struct NewsService {
    news_sources: HashSet<NewsSource>,
    polling_rate: u64,
}

impl NewsService {
    pub fn new() -> Self {
        let news_sources;
        let polling_rate;
        {
            let settings = SETTINGS.read().unwrap();
            news_sources = settings.news_settings.news_sources.clone();
            polling_rate = settings.news_settings.polling_rate as u64;
        }
        Self {
            news_sources,
            polling_rate,
        }
    }

    pub fn get_name() -> String {
        String::from("News")
    }
}

#[async_trait]
impl Service for NewsService {
    fn get_service_name(&self) -> String {
        NewsService::get_name()
    }

    fn get_polling_rate(&self) -> u64 {
        self.polling_rate
    }

    async fn update(&self) -> UpdateMessage {
        let mut news_list = Vec::new();
        for source in &self.news_sources {
            match source.get_news().await {
                Ok(mut news) => news_list.append(&mut news),
                Err(_) => continue,
            }
        }
        UpdateMessage::News(news_list)
    }
}

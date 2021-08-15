use crate::message::UpdateMessage;
use crate::{service::Service, settings::SETTINGS};
use actix_rt::time::interval;
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use futures::channel::mpsc;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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
    tx: Option<mpsc::Sender<Box<dyn erased_serde::Serialize + Send + Sync>>>,
}

impl NewsService {
    pub fn new() -> Self {
        Self { tx: None }
    }

    pub fn get_service_name() -> String {
        String::from("News")
    }
}

#[async_trait]
impl Service for NewsService {
    fn set_sender(&mut self, tx: mpsc::Sender<Box<dyn erased_serde::Serialize + Send + Sync>>) {
        self.tx = Some(tx);
    }

    async fn start_service(&mut self) {
        let news_sources;
        let polling_rate;
        {
            let settings = SETTINGS.read().unwrap();
            news_sources = settings.news_settings.news_sources.clone();
            polling_rate = settings.news_settings.polling_rate as u64;
        }
        let mut interval = interval(Duration::from_secs(polling_rate));
        loop {
            let mut news_list = Vec::new();
            for source in &news_sources {
                match source.get_news().await {
                    Ok(mut news) => news_list.append(&mut news),
                    Err(_) => continue,
                }
            }
            let news_message = UpdateMessage::News(news_list);
            if let Some(tx) = &mut self.tx {
                if tx.try_send(Box::new(news_message)).is_err() {
                    eprintln!("News receiver has been closed somehow.");
                }
            } else {
                eprintln!("News services not correctly initialized.");
            }
            interval.tick().await;
        }
    }

    fn get_service_name(&self) -> String {
        NewsService::get_service_name()
    }
}

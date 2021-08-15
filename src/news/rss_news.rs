use crate::news::NewsItem;
use chrono::DateTime;
use rss::Channel;
use serde::{Deserialize, Serialize};
use strum::ToString;

const NPR_FEED: &str = "http://www.npr.org/rss/rss.php?id=1001";
const BBC_FEED: &str = "http://newsrss.bbc.co.uk/rss/newsonline_world_edition/americas/rss.xml";

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq, ToString)]
#[allow(clippy::upper_case_acronyms)]
pub enum RssNewsSource {
    NPR,
    BBC,
    Custom(String, String),
}

impl RssNewsSource {
    fn get_url(&self) -> String {
        match self {
            RssNewsSource::NPR => NPR_FEED.into(),
            RssNewsSource::BBC => BBC_FEED.into(),
            RssNewsSource::Custom(_, url) => url.into(),
        }
    }

    pub async fn get_news(&self) -> Result<Vec<NewsItem>, Box<dyn std::error::Error>> {
        let url = self.get_url();
        let content = reqwest::get(url).await?.bytes().await?;
        let channel = Channel::read_from(&content[..])?;
        let source_name = match self {
            RssNewsSource::Custom(name, _) => name.clone(),
            _ => self.to_string(),
        };
        let news_items = channel
            .items
            .iter()
            .cloned()
            .map(|item| NewsItem {
                source: source_name.clone(),
                title: item.title,
                description: item.description,
                pub_date: item
                    .pub_date
                    .map(|date| DateTime::parse_from_rfc2822(&date).ok())
                    .flatten(),
            })
            .collect();
        Ok(news_items)
    }
}

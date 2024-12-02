use anyhow::Result;
use rss::Channel;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

#[derive(Debug, Serialize, Deserialize)]
struct NewsItem {
    title: String,
    link: String,
    pub_date: String,
    source: Option<String>,
    description: Option<String>,
}

struct GoogleNewsReader {
    client: reqwest::Client,
}

impl GoogleNewsReader {
    fn new() -> Self {
        GoogleNewsReader {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .build()
                .unwrap(),
        }
    }

    fn build_url(&self, query: Option<&str>) -> String {
        let base_url = "https://news.google.com/rss";
        match query {
            Some(q) => format!("{}/search?q={}&hl=ko&gl=KR&ceid=KR:ko", base_url, encode(q)),
            None => format!("{}?hl=ko&gl=KR&ceid=KR:ko", base_url),
        }
    }

    async fn fetch_news(&self, query: Option<&str>) -> Result<Vec<NewsItem>> {
        let url = self.build_url(query);
        println!("Fetching news from: {}", url);

        let content = self.client.get(&url).send().await?.bytes().await?;

        let channel = Channel::read_from(&content[..])?;

        let items: Vec<NewsItem> = channel
            .items()
            .iter()
            .map(|item| NewsItem {
                title: item.title().unwrap_or("").to_string(),
                link: item.link().unwrap_or("").to_string(),
                pub_date: item.pub_date().unwrap_or("").to_string(),
                source: item.source().map(|s| s.title().unwrap_or("").to_string()),
                description: item.description().map(String::from),
            })
            .collect();

        Ok(items)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let reader = GoogleNewsReader::new();

    // 특정 키워드로 검색
    let keyword = "해운";
    println!("\n{} 관련 뉴스:", keyword);
    let search_news = reader.fetch_news(Some(keyword)).await?;

    // 결과 출력
    for item in search_news {
        println!("\n제목: {}", item.title);
        if let Some(source) = item.source {
            println!("출처: {}", source);
        }
        println!("링크: {}", item.link);
        println!("날짜: {}", item.pub_date);
        if let Some(desc) = item.description {
            println!("설명: {}", desc);
        }
        println!("-------------------");
    }

    Ok(())
}

use dotenv::dotenv;
use futures::future::join_all;
use html_escape::decode_html_entities;
use rand::seq::SliceRandom;
use regex::Regex;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Error,
};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
struct NewsItem {
    description: String,
    link: String,
    #[serde(default)]
    originallink: String,
    pubDate: String,
    title: String,
}

#[derive(Debug, Deserialize)]
struct NewsResponse {
    total: i32,
    start: i32,
    display: i32,
    items: Vec<NewsItem>,
}

fn remove_html_tags(text: &str) -> String {
    // HTML 태그 제거
    let re = Regex::new(r"<[^>]*>").unwrap();
    let text_without_tags = re.replace_all(text, "");

    // HTML 엔티티 디코딩
    decode_html_entities(&text_without_tags).into_owned()
}

async fn fetch_news(
    client: &reqwest::Client,
    headers: &HeaderMap,
    keyword: &str,
) -> Result<NewsResponse, Error> {
    client
        .get("https://openapi.naver.com/v1/search/news.json")
        .headers(headers.clone())
        .query(&[("query", keyword), ("display", "15"), ("sort", "date")])
        .send()
        .await?
        .json()
        .await
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok(); // .env 파일 로드
    let client_id = env::var("NAVER_CLIENT_ID").expect("NAVER_CLIENT_ID not set");
    let client_secret = env::var("NAVER_CLIENT_SECRET").expect("NAVER_CLIENT_SECRET not set");

    let mut headers = HeaderMap::new();
    headers.insert(
        "X-Naver-Client-Id",
        HeaderValue::from_str(&client_id).unwrap(),
    );
    headers.insert(
        "X-Naver-Client-Secret",
        HeaderValue::from_str(&client_secret).unwrap(),
    );

    let keywords = vec![
        "해운",
        "선박",
        "컨테이너",
        "무역",
        "선적",
        "물류",
        "수출",
        "해양 운수",
        "중고차",
    ];
    let client = reqwest::Client::new();

    let futures: Vec<_> = keywords
        .iter()
        .map(|keyword| fetch_news(&client, &headers, keyword))
        .collect();

    let results = join_all(futures).await;

    let mut all_news: Vec<NewsItem> = Vec::new();
    for result in results {
        if let Ok(response) = result {
            all_news.extend(response.items);
        }
    }

    const NEWS_COUNT: usize = 20;

    let selected = all_news
        .choose_multiple(&mut rand::thread_rng(), NEWS_COUNT)
        .collect::<Vec<_>>();

    println!("\n무작위로 선택된 {}개의 뉴스:", NEWS_COUNT);
    println!("----------------------------------------");
    for item in selected {
        println!("- {}\n{}\n", remove_html_tags(&item.title), item.link);
    }
    println!("----------------------------------------");
    Ok(())
}

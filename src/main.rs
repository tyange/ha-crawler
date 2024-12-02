use dotenv::dotenv;
use futures::future::join_all;
use html_escape::decode_html_entities;
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
        .query(&[("query", keyword), ("display", "5"), ("sort", "date")])
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

    let keywords = vec!["해운", "선박", "컨테이너", "무역", "선적", "물류", "수출"];
    let client = reqwest::Client::new();

    // 모든 키워드에 대한 API 요청을 동시에 실행
    let futures: Vec<_> = keywords
        .iter()
        .map(|keyword| fetch_news(&client, &headers, keyword))
        .collect();

    let results = join_all(futures).await;

    // 각 키워드별 결과 출력
    for (keyword, result) in keywords.iter().zip(results) {
        match result {
            Ok(response) => {
                println!("\n검색어: {}", keyword);
                println!("----------------------------------------");
                for item in response.items {
                    println!("-{}\n{}", remove_html_tags(&item.title), item.link);
                }
                println!("----------------------------------------");
            }
            Err(e) => println!("키워드 '{}' 검색 중 오류 발생: {}", keyword, e),
        }
    }

    Ok(())
}

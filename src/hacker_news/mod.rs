use serde::{Serialize, Deserialize};
use futures::future;
use std::clone::Clone;
#[derive(Serialize, Deserialize, Debug)]
pub struct HNPost {
    pub title: String,
    pub score: u32,
    pub url: Option<String>,
    pub by: String,
    pub time: u64,
    pub id: u64,
    pub descendants: Option<u32>,
    pub kids: Option<Vec<u32>>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HNComment {
    pub id: u64,
    pub text: String,
    pub by: String
}


#[tokio::main]
pub async fn get_data(category: &str) -> Result<Vec<HNPost>, Box<dyn std::error::Error>> {
    let resp = reqwest::get("https://hacker-news.firebaseio.com/v0/topstories.json?print=pretty")
    .await?
    .json::<Vec<u32>>()
    .await?;

    let top_post_ids = &resp[..100];
    
    let all_posts = future::join_all(top_post_ids.into_iter().map( |post_id|  {
        async move {
            let resp = reqwest::get(format!("https://hacker-news.firebaseio.com/v0/item/{}.json", post_id)).await;
            let post = resp.unwrap().json::<HNPost>().await;
            post.unwrap()
        }
    })).await;
    Ok(all_posts)
}

#[tokio::main]
pub async fn get_comments(kids: Vec<u32>) -> Result<Vec<HNComment>, Box<dyn std::error::Error>> {

    let all_comments = future::join_all(kids.into_iter().map(|kid| {
        async move {
            let resp = reqwest::get(format!("https://hacker-news.firebaseio.com/v0/item/{}.json", kid)).await;
            let comment = resp.unwrap().json::<HNComment>().await;
            comment.unwrap()
        }
    })).await;

    Ok(all_comments)
}

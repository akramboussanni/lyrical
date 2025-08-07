use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use reqwest::blocking::Client;
use std::error::Error;

const SEARCH_URL: &str = "https://lrclib.net/api/search";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub id: i64,
    pub track_name: String,
    pub artist_name: String,
    pub album_name: String,
    pub duration: f64,
    pub instrumental: bool,
    pub synced_lyrics: Option<String>,
}

pub fn request(params: HashMap<String, String>) -> Result<Vec<Response>, Box<dyn Error>> {
    if params.contains_key("q") && params.contains_key("track_name") {
        return Err("Warning: 'q' overrides 'track_name'.".into());
    }

    let client = Client::new();
    let body = client
        .get(SEARCH_URL)
        .query(&params)
        .send()?
        .text()?;

    let resp = serde_json::from_str::<Vec<Response>>(&body)?;
    Ok(resp)
}
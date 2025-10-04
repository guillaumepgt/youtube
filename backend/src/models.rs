use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Video {
    pub url: String,
    pub video_id: String,
    pub published_at: DateTime<Utc>,
    pub title: String,
    pub thumbnail: String,
    pub channel_title: String,
}

#[derive(Serialize, Deserialize)]
pub struct SavedToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub issued_at: DateTime<Utc>,
}
use axum::{debug_handler, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    html::{HomepageTemplate, HtmlTemplate},
    nostr::RelayHandler,
};

pub async fn homepage() -> impl IntoResponse {
    HtmlTemplate(HomepageTemplate {})
}

#[derive(Deserialize, Serialize)]
pub struct ContactMeRequest {
    pub name: String,
    pub contact: String,
    pub message: String,
}

impl ContactMeRequest {
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

pub async fn contact_me(post: Json<ContactMeRequest>) -> Result<(), String> {
    if let Some(relay_handler) = RelayHandler::new() {
        match relay_handler
            .send_contact_me_note(&post.to_json_string())
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    } else {
        Err("Could not connect to relay".to_string())
    }
}

pub async fn forward_notes_to_telegram() -> Result<(), String> {
    if let Some(relay_handler) = RelayHandler::new() {
        info!("Reading messages");
        match relay_handler.read_notes().await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    } else {
        Err("Could not connect to relay".to_string())
    }
}

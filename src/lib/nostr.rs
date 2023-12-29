use nostro2::{
    notes::Note,
    relays::{NostrRelay, RelayEvents},
    userkeys::UserKeys,
};
use openssl::ec::EcKey;
use serde_json::json;
use teloxide::{
    requests::{Request, Requester},
    Bot,
};
use tracing::info;

use crate::handlers::ContactMeRequest;

pub struct RelayHandler {
    relay_url: String,
    private_key: UserKeys,
    channel_id: String,
}

impl RelayHandler {
    pub fn new() -> Option<Self> {
        let relay_url = std::env::var("RELAY_URL");
        let channel_id = std::env::var("CHANNEL_ID");
        let pem_file = std::fs::read("website.pem");
        if relay_url.is_err() || pem_file.is_err() || channel_id.is_err() {
            return None;
        };

        let buffer = EcKey::private_key_from_pem(&pem_file.unwrap());

        if buffer.is_err() {
            return None;
        }

        let private_key = UserKeys::new(&buffer.unwrap().private_key().to_hex_str().unwrap());

        if private_key.is_err() {
            return None;
        }

        Some(Self {
            relay_url: relay_url.unwrap(),
            private_key: private_key.unwrap(),
            channel_id: channel_id.unwrap(),
        })
    }

    pub async fn send_contact_me_note(&self, message: &str) -> Result<(), String> {
        if let Ok(relay) = NostrRelay::new(&self.relay_url).await {
            let new_note = Note::new(self.private_key.get_public_key(), 4242, message);
            let signed_note = self
                .private_key
                .sign_encrypted_nostr_event(new_note, self.private_key.get_public_key());
            let sent = relay.send_note(signed_note).await;
            match sent {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Could not send note: {:?}", e)),
            }
        } else {
            Err("Could not connect to relay".to_string())
        }
    }

    pub async fn read_notes(&self) -> Result<(), String> {
        let bot = Bot::from_env();
        if let Ok(relay) = NostrRelay::new(&self.relay_url).await {
            let filter = json!({
                "kinds": [4242],
                "authors": [self.private_key.get_public_key()],
                "since": nostro2::utils::get_unix_timestamp() - 60,
            });

            let subscribed = relay.subscribe(filter).await;

            if subscribed.is_err() {
                return Err("Could not subscribe to relay".to_string());
            }

            while let Some(Ok(events)) = relay.read_from_relay().await {
                match events {
                    RelayEvents::EVENT(_e, _id, note) => {
                        if let Ok(plaintext) = self.private_key.decrypt_note_content(&note) {
                            match bot
                                .send_message(
                                    self.channel_id.clone(),
                                    Self::format_telegram_message(&plaintext),
                                )
                                .send()
                                .await
                            {
                                Ok(_) => info!("Message sent"),
                                Err(e) => info!("Error sending message: {:?}", e),
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(())
        } else {
            Err("Could not connect to relay".to_string())
        }
    }

    fn format_telegram_message(plaintext: &str) -> String {
        if let Ok(contact_me) = serde_json::from_str::<ContactMeRequest>(plaintext) {
            format!(
                "Jaque!\n\n{}, {}\n{}",
                contact_me.name, contact_me.contact, contact_me.message
            )
        } else {
            format!("Nuevo mensaje:\n\n{}", plaintext)
        }
    }
}

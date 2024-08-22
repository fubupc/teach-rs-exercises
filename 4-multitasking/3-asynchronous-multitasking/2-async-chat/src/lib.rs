use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    /// A user enters the chat and provides their username
    User(String),
    /// A message sent from a client,
    /// that needs to be matched with their username
    ClientMessage(String),
    /// A message sent from the server to the clients,
    /// containing the username of the sender and the message content
    Chat { user: String, content: String },
}

pub fn serialize_message(msg: Message) -> Result<Vec<u8>> {
    let mut json = serde_json::to_vec(&msg)?;
    json.push(b'\n');
    Ok(json)
}

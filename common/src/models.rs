use rsa::RsaPublicKey;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct EntryInfo {
    pub uuid: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct NewUser {
    pub name: String,
    pub password: String,
    pub public_key: RsaPublicKey,
}

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    pub name: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Deserialize, Serialize)]
pub struct NewUpdate {
    pub aes_key: Vec<u8>,
    pub signature: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateInfo {
    pub id: Uuid,
    pub created: chrono::NaiveDateTime,
    pub aes_key: Vec<u8>,
    pub signature: Vec<u8>,
}

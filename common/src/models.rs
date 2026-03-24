use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct EntryInfo {
    pub uuid: Uuid,
}

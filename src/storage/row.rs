use crate::storage::constant::ROW_SIZE;
use anyhow::Result;
use serde::{Deserialize, Serialize};

//todo: enforce char count for the string smh
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Row {
    id: u32,
    email: String,
    user_name: String,
}

impl Row {
    pub fn new(id: u32, user_name: String, email: String) -> Self {
        Self {
            id,
            email,
            user_name,
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_email(&self) -> &str {
        &self.email
    }

    pub fn get_user_name(&self) -> &str {
        &self.user_name
    }

    //todo: custom serializer and deserializer are a pain for the time being, so using this hack for now
    pub fn serialize(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|err| anyhow::Error::msg(err))
            .map(|mut bytes| {
                bytes.resize(ROW_SIZE, 0);
                bytes
            })
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Row> {
        let offset = bytes[0..ROW_SIZE]
            .iter()
            .position(|&byte| byte == 0u8)
            .unwrap_or(ROW_SIZE);
        serde_json::from_slice(&bytes[0..offset]).map_err(|err| anyhow::Error::msg(err))
    }
}

use crate::storage::constant::{EMAIL_OFFSET, ID_OFFSET, USER_NAME_OFFSET};
use crate::storage::constant::{EMAIL_SIZE, ID_SIZE, ROW_SIZE, USER_NAME_SIZE};
use anyhow::Result;
use std::convert::TryInto;

//todo: enforce char count for the string smh
#[derive(Debug)]
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

    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend(&self.id.to_ne_bytes());
        bytes.extend(Self::serialize_string(&self.email, EMAIL_SIZE));
        bytes.extend(Self::serialize_string(&self.user_name, USER_NAME_SIZE));

        Ok(bytes)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Row> {
        if bytes.len() != ROW_SIZE {
            return Err(anyhow::Error::msg("Bytes size doesnt match target size."));
        }

        // we serialize as ne_bytes, probably should deserialize as well in ne
        let id = u32::from_ne_bytes(
            bytes[ID_OFFSET..ID_SIZE]
                .try_into()
                .expect("Error when converting bytes to id"),
        );

        //todo: problems with order might arise, need to think how to bind it to fields order
        let email = Self::deserialize_string(bytes, EMAIL_OFFSET, EMAIL_SIZE);
        let user_name = Self::deserialize_string(bytes, USER_NAME_OFFSET, USER_NAME_SIZE);

        Ok(Self::new(id, user_name, email))
    }

    fn serialize_string(str: &String, target_sie: usize) -> Vec<u8> {
        let mut str_bytes = str.clone().into_bytes();
        str_bytes.resize(target_sie, 0);
        str_bytes
    }

    fn deserialize_string(bytes: &[u8], position_start: usize, target_size: usize) -> String {
        let offset = bytes[position_start..position_start + target_size]
            .iter()
            .position(|&byte| byte == 0u8)
            .unwrap_or(target_size);

        String::from_utf8(bytes[position_start..position_start + offset].to_vec())
            .expect("Invalid UTF-8 sequence in byte array.")
    }
}

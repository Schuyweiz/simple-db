const TABLE_MAX_PAGES: usize = 100;
const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;

pub struct Table {
    current_rows: usize,
    pages: Vec<Option<Page>>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            current_rows: 0,
            pages: vec![None; TABLE_MAX_PAGES],
        }
    }

    pub fn get_page_mut(&mut self, page_num: usize) -> &mut Page {
        if self.pages[page_num].is_none() {
            self.pages[page_num] = Some(Page::new());
        }

        self.pages[page_num].as_mut().unwrap()
    }

    pub fn get_page_ref(&self, page_num: usize) -> Option<&Page> {
        self.pages[page_num].as_ref()
    }

    pub fn increment_current_row_count(&mut self) {
        self.current_rows += 1;
    }

    pub fn get_current_row_count(&self) -> usize {
        self.current_rows
    }
}


const PAGE_SIZE: usize = 4096;

#[derive(Clone, Copy, Debug)]
pub struct Page {
    data: [u8; PAGE_SIZE],
}

impl Page {
    fn new() -> Page {
        Page { data: [0; PAGE_SIZE] }
    }

    pub fn get_mut_slot(&mut self, row_index: usize) -> Vec<u8> {
        let page_offset = row_index % ROWS_PER_PAGE;
        let byte_offset = page_offset * ROW_SIZE;
        self.data[byte_offset..byte_offset + ROW_SIZE].to_vec()
    }

    pub fn read_slot(&self, row_index: usize) -> Vec<u8> {
        let page_offset = row_index % ROWS_PER_PAGE;
        let byte_offset = page_offset * ROW_SIZE;

        self.data[byte_offset..byte_offset + ROW_SIZE].to_vec()
    }

    pub fn write_slot(&mut self, row_index: usize, bytes: &[u8]) {
        let page_offset = row_index % ROWS_PER_PAGE;
        let byte_offset = page_offset * ROW_SIZE;

        self.data[byte_offset..byte_offset + ROW_SIZE].copy_from_slice(bytes);
    }
}


const USER_NAME_SIZE: usize = 32;
const EMAIL_SIZE: usize = 255;
const ID_SIZE: usize = std::mem::size_of::<u32>();
const ROW_SIZE: usize = USER_NAME_SIZE + EMAIL_SIZE + ID_SIZE;
const ID_OFFSET: usize = 0;
const EMAIL_OFFSET: usize = ID_OFFSET + ID_SIZE;
const USER_NAME_OFFSET: usize = EMAIL_OFFSET + EMAIL_SIZE;

use anyhow::Result;

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
            user_name,
            email,
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

        let id = u32::from_ne_bytes(
            bytes[0..ID_SIZE]
                .try_into()
                .expect("Error when converting bytes to id")
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

        String::from_utf8(
            bytes[position_start..position_start + offset].to_vec())
            .expect("Invalid UTF-8 sequence in byte array."
            )
    }
}

use crate::storage::constant::{PAGE_SIZE, ROWS_PER_PAGE, ROW_SIZE, TABLE_MAX_PAGES};
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Read, Seek, Write};

pub struct Pager {
    file: File,
    rows_count: usize,
    pages: Vec<Option<Page>>,
}

impl Pager {
    pub fn new(file_path: &str) -> anyhow::Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_path)
            .unwrap_or_else(|err| panic!("Failed to open file {} {:?}", file_path, err));

        let valid_rows = Self::calculate_valid_rows(&mut file)?;

        Ok(Self {
            file,
            rows_count: valid_rows,
            pages: vec![None; TABLE_MAX_PAGES],
        })
    }

    pub fn get_page_mut(&mut self, page_num: usize) -> &mut Page {
        if self.pages[page_num].is_none() {
            //cache miss
            Self::load_page_from_file(self, page_num);
        }

        self.pages[page_num].as_mut().unwrap()
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        for i in 0..self.pages.len() {
            let page = self.pages[i];

            match page {
                None => {
                    continue;
                }
                Some(page) => {
                    self.file
                        .seek(io::SeekFrom::Start((i * PAGE_SIZE) as u64))
                        .unwrap();
                    self.file.write(&page.data).unwrap();
                    self.file.flush().unwrap();
                }
            }
        }

        Ok(())
    }

    pub fn get_rows_count(&self) -> usize {
        self.rows_count
    }

    fn load_page_from_file(&mut self, page_num: usize) {
        if page_num <= self.rows_count {
            self.file
                .seek(io::SeekFrom::Start((page_num * PAGE_SIZE) as u64))
                .unwrap();
            let mut buffer = vec![0; PAGE_SIZE];
            self.file.read(&mut buffer).unwrap();
            self.pages[page_num] = Some(Page::deserialize(&buffer));
        }
    }

    fn calculate_valid_rows(file: &mut File) -> io::Result<usize> {
        let mut valid_rows = 0;
        let mut buffer = [0u8; PAGE_SIZE];

        loop {
            match file.read(&mut buffer) {
                Ok(0) => break, // End of file
                Ok(bytes_read) => {
                    // Calculate the number of complete rows in the read buffer
                    let num_rows = bytes_read / ROW_SIZE;

                    // Process each row in the page
                    for i in 0..num_rows {
                        let row_start = i * ROW_SIZE;
                        let row = &buffer[row_start..row_start + ROW_SIZE];
                        if row.iter().any(|&byte| byte != 0) {
                            valid_rows += 1;
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Ok(valid_rows)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Page {
    data: [u8; PAGE_SIZE],
}

impl Page {
    fn new() -> Page {
        Page {
            data: [0; PAGE_SIZE],
        }
    }

    fn deserialize(data: &[u8]) -> Page {
        let mut page = Page::new();
        page.data.copy_from_slice(data);
        page
    }

    pub fn read_from_slot(&self, row_index: usize) -> &[u8] {
        let page_offset = row_index % ROWS_PER_PAGE;
        let byte_offset = page_offset * ROW_SIZE;

        &self.data[byte_offset..byte_offset + ROW_SIZE]
    }

    pub fn write_to_slot(&mut self, row_index: usize, bytes: &[u8]) {
        let page_offset = row_index % ROWS_PER_PAGE;
        let byte_offset = page_offset * ROW_SIZE;

        self.data[byte_offset..byte_offset + ROW_SIZE].copy_from_slice(bytes);
    }
}

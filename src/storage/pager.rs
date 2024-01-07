use crate::storage::constant::{PAGE_SIZE, ROW_SIZE, TABLE_MAX_PAGES};
use crate::storage::node::Node;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Read, Seek, Write};

pub struct Pager {
    file: File,
    nodes_count: usize,
    nodes: Vec<Option<Node>>,
}

impl Pager {
    pub fn new(file_path: &str) -> anyhow::Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_path)
            .unwrap_or_else(|err| panic!("Failed to open file {} {:?}", file_path, err));

        let file_len = file.metadata()?.len();
        let pages_count = (file_len / PAGE_SIZE as u64) as usize;

        Ok(Self {
            file,
            nodes_count: pages_count,
            nodes: vec![None; TABLE_MAX_PAGES],
        })
    }

    pub fn get_page_count(&self) -> usize {
        self.nodes_count
    }

    pub fn insert(&mut self, key: &[u8], value: &[u8], page_num: usize) -> anyhow::Result<()> {
        let mut node = self.get_node_mut(page_num);
        node.insert_key_value(key, value);
        Ok(())
    }

    pub fn select(&mut self, page_num: usize, cell_num: usize) -> &[u8] {
        let node = self.get_node_mut(page_num);
        node.get_value(cell_num)
    }

    pub fn get_node_mut(&mut self, page_num: usize) -> &mut Node {
        if self.nodes[page_num].is_none() {
            //cache miss
            Self::load_page_from_file(self, page_num);
        }

        self.nodes[page_num].as_mut().unwrap()
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        for i in 0..self.nodes.len() {
            let node = self.nodes[i].take();

            match node {
                None => {
                    continue;
                }
                Some(node) => {
                    self.file
                        .seek(io::SeekFrom::Start((i * PAGE_SIZE) as u64))
                        .unwrap();
                    self.file.write(&node.serialize()).unwrap();
                    self.file.flush().unwrap();
                }
            }
        }

        Ok(())
    }

    fn load_page_from_file(&mut self, page_num: usize) {
        if page_num <= self.nodes_count {
            self.file
                .seek(io::SeekFrom::Start((page_num * PAGE_SIZE) as u64))
                .unwrap();
            let mut buffer = vec![0; PAGE_SIZE];
            self.file.read(&mut buffer).unwrap();
            self.nodes[page_num] = Some(Node::deserialize(&buffer));
            self.nodes_count += 1;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::storage::row::Row;
    use std::{fs, panic};

    #[test]
    fn test_pager() {
        let test_db_path = "test.db";

        let result = panic::catch_unwind(|| {
            let mut pager = Pager::new(test_db_path).unwrap();
            let mut node = pager.get_node_mut(0);
            let row = Row::new(1, "hello world".to_string(), "hello world".to_string());
            node.insert_key_value(
                row.get_id().to_le_bytes().as_mut(),
                &row.serialize().unwrap(),
            );

            pager.flush().unwrap();
            let mut pager = Pager::new(test_db_path).unwrap();
            let node = pager.get_node_mut(0);
            let row = Row::deserialize(node.get_value(0)).unwrap();
            assert_eq!(row.get_id(), 1);
            assert_eq!(row.get_user_name(), "hello world");

            fs::remove_file(test_db_path).expect("Failed to remove test database file");
        });

        // Re-panic if the test failed
        assert!(result.is_ok());
    }
}

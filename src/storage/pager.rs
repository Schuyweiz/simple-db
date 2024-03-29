use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Read, Seek, Write};

use crate::storage::constant::{ID_SIZE, INTERNAL_CELL_SIZE, LEAF_NODE_MAX_CELLS, PAGE_SIZE, TABLE_MAX_PAGES};
use crate::storage::node::{InternalCell, Node, NodeType};

pub struct Pager {
    file: File,
    nodes_count: usize,
    nodes: Vec<Option<Node>>,
}

impl Pager {
    pub fn new(file_path: &str) -> anyhow::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_path)
            .unwrap_or_else(|err| panic!("Failed to open file {} {:?}", file_path, err));

        let file_len = file.metadata()?.len();
        let nodes_count = (file_len / PAGE_SIZE as u64) as usize;

        Ok(Self {
            file,
            nodes_count,
            nodes: vec![None; TABLE_MAX_PAGES],
        })
    }

    pub fn print_tree(&self, page_num: usize, indentation: usize) {
        if let Some(node) = &self.nodes[page_num] {
            let indent = " ".repeat(indentation);
            println!("{}Node Type: {:?}", indent, node.node_type);
            println!("{}Is Root: {}", indent, node.is_root);
            println!("{}Parent Page Num: {}", indent, node.parent_page_num);

            match node.node_type {
                NodeType::Leaf => {
                    println!("{}Next Leaf Num: {}", indent, node.next_leaf_num);
                    println!("{}Cells Count: {}", indent, node.cells_count);
                    for cell in &node.cells {
                        println!("{}Cell: {:?}", indent, cell.get_key());
                    }
                }
                NodeType::Internal => {
                    println!("{}Right Child Key: {}", indent, node.right_child_key);
                    println!("{}Keys Count: {}", indent, node.keys_count);
                    for key in &node.keys {
                        println!("{}Key: {:?}", indent, key);
                    }
                    // Recursively print child nodes
                    for key in &node.keys {
                        self.print_tree(
                            key.get_key(),
                            indentation + 4,
                        );
                    }
                    self.print_tree(node.right_child_key, indentation + 4);
                }
            }
        }
    }

    pub fn get_empty_page_num(&mut self) -> usize {
        // we do not free old pages yet, so this is always the next page
        self.nodes_count
    }

    pub fn get_page_count(&self) -> usize {
        self.nodes_count
    }

    //todo: get rid of the page notation if possible, seems to have no use here
    pub fn insert(&mut self, key: usize, value: &[u8], node_index: usize, cell_index: usize) -> anyhow::Result<()> {
        let (current_node_max_key, current_node_cell_count, node_parent_page_num) = {
            let current_node = self.get_node_mut(node_index);
            (current_node.get_node_max_key(), current_node.get_cell_count(), current_node.get_parent_page_num())
        };

        if LEAF_NODE_MAX_CELLS <= current_node_cell_count {
            let right_child_page_num = self.leaf_node_split_and_insert(node_index, cell_index, key, value);

            if node_parent_page_num == node_index {
                Self::create_new_root_node(self, node_index, right_child_page_num);
            } else {
                let new_max_key = self.get_node_mut(node_index).get_node_max_key();
                let mut parent_node = self.get_node_mut(node_parent_page_num);
                parent_node.update_internal_node_key(current_node_max_key, new_max_key);
                Self::internal_node_insert(self, node_parent_page_num, right_child_page_num);
            }
            return Ok(());
        }

        self.get_node_mut(node_index).insert_key_value(key, value, cell_index);
        Ok(())
    }

    fn internal_node_insert(&mut self, parent_page_num: usize, new_page_num: usize) {
        let (new_node_max_key) = {
            let new_node = self.get_node_mut(new_page_num);
            (new_node.get_node_max_key())
        };

        let (new_child_index, original_parent_keys_num, parent_right_child_key) = {
            let parent_node = self.get_node_mut(parent_page_num);
            let new_child_index = parent_node.internal_find_child_index_by_key(new_node_max_key);
            let original_parent_keys_num = parent_node.get_key_count();
            let parent_right_child_key = parent_node.right_child_key;
            (new_child_index, original_parent_keys_num, parent_right_child_key)
        };

        if original_parent_keys_num >= LEAF_NODE_MAX_CELLS {
            panic!("Need to implement internal node split")
        }

        let right_child_node = self.get_node_mut(parent_right_child_key);
        let right_child_node_max_key = right_child_node.get_node_max_key();

        let parent_node = self.get_node_mut(parent_page_num);

        if new_node_max_key > right_child_node_max_key {
            parent_node.internal_node_insert(right_child_node_max_key, parent_right_child_key);
            parent_node.set_right_child_key(new_child_index)
        } else {
            parent_node.internal_node_insert_by_index(new_node_max_key, new_page_num, new_child_index)
        }
    }


    fn create_new_root_node(&mut self, root_page_num: usize, right_child_page_num: usize) {
        let root_node_cell_count;
        let last_key: usize;
        let mut left_child_node = Node::new_leaf();

        {
            let root_node = self.get_node_mut(root_page_num);
            root_node_cell_count = root_node.get_cell_count();
            for cell_id in 0..root_node_cell_count {
                left_child_node.insert_cell(root_node.get_mut_cell(cell_id).clone(), cell_id);
            }
            last_key = root_node.get_key(root_node_cell_count - 1);
        }

        left_child_node.set_parent_page_num(root_page_num);
        left_child_node.set_next_leaf_num(right_child_page_num);

        let mut new_root_node = Node::new_internal();
        new_root_node.set_parent_page_num(root_page_num);

        Self::set_node(self, left_child_node, self.nodes_count);

        new_root_node.set_is_root(true);
        new_root_node.set_right_child_key(right_child_page_num);
        new_root_node.internal_node_insert(
            last_key,
            self.nodes_count,
        );

        self.nodes_count += 1;
        self.nodes[root_page_num] = Some(new_root_node);
    }

    fn set_node(&mut self, node: Node, page_num: usize) {
        self.nodes[page_num] = Some(node);
    }

    // returning usize which is the new page num, but looks like a temp hack to me
    fn leaf_node_split_and_insert(&mut self, page_num: usize, cell_num: usize, key: usize, value: &[u8]) -> usize {
        let nodes_count = self.nodes_count;
        let mut new_node = Node::new_leaf();
        let mut old_node = self.get_node_mut(page_num);
        new_node.set_parent_page_num(page_num);
        new_node.set_next_leaf_num(old_node.get_next_leaf_num());
        old_node.set_next_leaf_num(nodes_count);

        Self::redistribute_cells(&mut old_node, &mut new_node);
        let old_node_cell_count = old_node.get_cell_count();


        if cell_num <= old_node_cell_count {
            old_node.insert_key_value(key, value, cell_num);
        } else {
            new_node.insert_key_value(key, value, cell_num - old_node_cell_count);
        }

        Self::append_new_node(self, new_node);

        // new page number
        nodes_count
    }

    fn append_new_node(&mut self, node: Node) {
        self.nodes[self.nodes_count] = Some(node);
        self.nodes_count += 1;
    }

    fn redistribute_cells(split_old_node: &mut Node, split_new_node: &mut Node) {
        let split_old_node_cell_count = split_old_node.get_cell_count();

        let old_node_new_max_cells = split_old_node_cell_count / 2;

        for _ in old_node_new_max_cells..split_old_node_cell_count {
            let cell = split_old_node.get_mut_cell(old_node_new_max_cells).clone();
            split_new_node.push_cell(cell);
            split_old_node.remove_cell(old_node_new_max_cells);
        }
    }

    pub fn select(&mut self, page_num: usize, cell_num: usize) -> &[u8] {
        let node = self.get_node_mut(page_num);
        node.get_value(cell_num)
    }

    pub fn get_key(&mut self, page_num: usize, cell_num: usize) -> usize {
        let node = self.get_node_mut(page_num);
        node.get_key(cell_num)
    }

    pub fn get_node(&self, page_num: usize) -> Option<&Node> {
        self.nodes[page_num].as_ref()
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
    use std::{fs, panic};

    use crate::storage::row::Row;

    use super::*;

    // #[test]
    // fn test_pager() {
    //     let test_db_path = "test.db";
    //
    //     let result = panic::catch_unwind(|| {
    //         let mut pager = Pager::new(test_db_path).unwrap();
    //         let mut node = pager.get_node_mut(0);
    //         let row = Row::new(1, "hello world".to_string(), "hello world".to_string());
    //         node.insert_key_value(&row.get_id().to_le_bytes(), &row.serialize().unwrap(), 0);
    //
    //         pager.flush().unwrap();
    //         let mut pager = Pager::new(test_db_path).unwrap();
    //         let node = pager.get_node_mut(0);
    //         let row = Row::deserialize(node.get_value(0)).unwrap();
    //         assert_eq!(row.get_id(), 1);
    //         assert_eq!(row.get_user_name(), "hello world");
    //
    //         fs::remove_file(test_db_path).expect("Failed to remove test database file");
    //     });
    //
    //     // Re-panic if the test failed
    //     assert!(result.is_ok());
    // }
    //
    // #[test]
    // fn test_node_split() {
    //     let test_db_path = "test.db";
    //
    //     let mut pager = Pager::new(test_db_path).unwrap();
    //     let row = Row::new(1, "hello world".to_string(), "hello world".to_string());
    //     pager.insert(&(row.get_id() as usize).to_le_bytes(), &row.serialize().unwrap(), 0, 0).unwrap();
    //     let row = Row::new(2, "hello world".to_string(), "hello world".to_string());
    //     pager.insert(&(row.get_id() as usize).to_le_bytes(), &row.serialize().unwrap(), 0, 1).unwrap();
    //     let row = Row::new(3, "hello world".to_string(), "hello world".to_string());
    //     pager.insert(&(row.get_id() as usize).to_le_bytes(), &row.serialize().unwrap(), 0, 2).unwrap();
    //     let row = Row::new(4, "hello world".to_string(), "hello world".to_string());
    //     pager.insert(&(row.get_id() as usize).to_le_bytes(), &row.serialize().unwrap(), 0, 3).unwrap();
    // }
}

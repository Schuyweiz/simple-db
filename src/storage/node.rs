use std::fmt::{Display, Formatter};
use std::usize;

use crate::storage::constant::{ID_SIZE, INTERNAL_CELL_SIZE, INTERNAL_NODE_MAX_CELLS, KEY_VALUE_OFFSET, KEY_VALUE_SIZE, PAGE_NUM_SIZE, PAGE_SIZE, RIGHT_CHILD_OFFSET};
// unused imports will be kept until the end of the project to know if they are really unused
use crate::storage::constant::{
    CELL_SIZE, CELLS_COUNT_OFFSET, CELLS_COUNT_SIZE, IS_ROOT_OFFSET,
    LEAF_NODE_CELLS_OFFSET, LEAF_NODE_MAX_CELLS,
    NODE_TYPE_OFFSET, PARENT_PAGE_NUM_OFFSET, PARENT_PAGE_NUM_SIZE
    ,
};

// replaced Page from previous implementation. Page structure will be restored later on if deemed necessary
#[derive(Clone, PartialEq)]
pub struct Node {
    // meta, common
    //todo: should be a separate struct perhaps?
    pub(crate) node_type: NodeType,
    pub(crate) is_root: bool,
    pub(crate) parent_page_num: usize,

    //meta leaf node
    //need to be here for manual deserialization without billion of rows with 0 values
    pub(crate) cells_count: usize,
    pub(crate) cells: Vec<Cell>,
    pub(crate) next_leaf_num: usize,

    //meta internal node
    pub(crate) keys_count: usize,
    pub(crate) keys: Vec<InternalCell>,
    pub(crate) right_child_key: usize,
}

impl InternalCell {
    pub fn get_key(&self) -> usize {
        let key_bytes = &self.0[..ID_SIZE];
        usize::from_le_bytes(key_bytes.try_into().unwrap())
    }

    pub fn get_page_num(&self) -> usize {
        let page_num_bytes = &self.0[ID_SIZE..];
        usize::from_le_bytes(page_num_bytes.try_into().unwrap())
    }
}

impl Cell {
    pub fn get_key(&self) -> usize {
        let key_bytes = &self.0[..ID_SIZE];
        usize::from_le_bytes(key_bytes.try_into().unwrap())
    }

    pub fn get_value(&self) -> &[u8] {
        &self.0[ID_SIZE..]
    }
}

impl Node {
    pub fn print_nodes(&self) {
        println!("Node type: {:?}", self.node_type);
        println!("Is root: {:?}", self.is_root);
        println!("Parent page num: {:?}", self.parent_page_num);
        println!("Cells count: {:?}", self.cells_count);
    }
    pub fn new_leaf() -> Node {
        Node {
            node_type: NodeType::Leaf,
            is_root: false,
            parent_page_num: 0,
            cells_count: 0,
            cells: Vec::new(),
            next_leaf_num: 0,

            keys_count: 0,
            keys: Vec::new(),
            right_child_key: 0,
        }
    }

    pub fn new_internal() -> Node {
        Node {
            node_type: NodeType::Internal,
            is_root: false,
            parent_page_num: 0,
            cells_count: 0,
            cells: Vec::new(),

            keys_count: 0,
            keys: Vec::new(),
            right_child_key: 0,
            next_leaf_num: 0,
        }
    }

    pub fn get_next_leaf_num(&self) -> usize {
        self.next_leaf_num
    }

    pub fn set_next_leaf_num(&mut self, next_leaf_num: usize) {
        self.next_leaf_num = next_leaf_num;
    }

    pub fn internal_node_children(&self, cell_num: usize) -> usize {
        if cell_num > self.keys_count {
            panic!("Tried to access child_num {} > keys_count {}", cell_num, self.keys_count);
        } else if cell_num == self.keys_count {
            return self.right_child_key;
        } else {
            return self.internal_get_value(cell_num);
        }
    }

    pub fn insert_cell(&mut self, cell: Cell, cell_num: usize) {
        if self.cells_count >= LEAF_NODE_MAX_CELLS {
            panic!("Trying to insert cell into a full leaf node");
        }

        // i really dont like this, but file deser requires cells_count to work
        // need a better way to serialzie cells to solve this one.
        self.cells.insert(cell_num, cell);
        self.cells_count += 1;
    }

    pub fn push_cell(&mut self, cell: Cell) {
        if self.cells_count >= LEAF_NODE_MAX_CELLS {
            panic!("Trying to insert cell into a full leaf node");
        }

        // i really dont like this, but file deser requires cells_count to work
        // need a better way to serialzie cells to solve this one.
        self.cells.push(cell);
        self.cells_count += 1;
    }

    pub fn insert_key_value(&mut self, key: usize, value: &[u8], cell_num: usize) {
        let mut cell = [0; CELL_SIZE];
        cell[..ID_SIZE].copy_from_slice(&key.to_le_bytes());
        cell[ID_SIZE..].copy_from_slice(&value);
        self.insert_cell(Cell(cell), cell_num);
    }

    pub fn leaf_get_index_by_key(&self, key: usize) -> Option<usize> {
        let mut min_index = 0;
        let mut max_index = self.cells_count;

        while min_index != max_index {
            let index = (min_index + max_index) / 2;
            let key_at_index = self.get_key(index);

            if key_at_index == key {
                return Some(index);
            } else if key_at_index < key {
                min_index = index + 1;
            } else {
                max_index = index;
            }
        }

        return None;
    }

    pub fn internal_find_child_index_by_key(&self, key: usize) -> usize {
        let mut min_index = 0;
        let mut max_index = self.keys_count;

        while min_index != max_index {
            let index = (min_index + max_index) / 2;
            let key_at_index = self.internal_get_key(index);

            if key_at_index == key {
                return index;
            } else if key_at_index < key {
                min_index = index + 1;
            } else {
                max_index = index;
            }
        }

        return min_index;
    }

    pub fn update_internal_node_key(&mut self, key: usize, new_key: usize) {
        let index = Self::internal_find_child_index_by_key(self, key);

        if index < self.keys_count {
            let mut key = [0; INTERNAL_CELL_SIZE];
            key[..ID_SIZE].copy_from_slice(&new_key.to_le_bytes());
            key[ID_SIZE..].copy_from_slice(&self.internal_get_value(index).to_le_bytes());
            self.keys[index] = InternalCell(key);
        }
    }

    fn leaf_get_node_max_key(&self) -> usize {
        if self.cells_count == 0 {
            return usize::MAX;
        }
        self.get_key(self.cells_count - 1)
    }

    fn internal_get_node_max_key(&self) -> usize {
        if self.keys_count == 0 {
            return usize::MAX;
        }
        self.internal_get_key(self.keys_count - 1)
    }

    pub fn get_node_max_key(&self) -> usize {
        match self.node_type {
            NodeType::Leaf => self.leaf_get_node_max_key(),
            NodeType::Internal => self.internal_get_node_max_key(),
        }
    }

    pub fn is_parent_node(&self) -> bool {
        self.node_type == NodeType::Internal
    }

    pub fn get_node_type(&self) -> NodeType {
        self.node_type.clone()
    }

    pub fn get_cell_count(&self) -> usize {
        self.cells_count
    }

    pub fn get_key_count(&self) -> usize {
        self.keys_count
    }

    pub fn set_parent_page_num(&mut self, parent_page_num: usize) {
        self.parent_page_num = parent_page_num;
    }

    pub fn set_is_root(&mut self, is_root: bool) {
        self.is_root = is_root;
    }

    pub fn set_right_child_key(&mut self, right_child_key: usize) {
        self.right_child_key = right_child_key;
    }

    pub fn internal_node_insert_by_index(&mut self, key: usize, value: usize, index: usize) {
        if self.keys_count >= INTERNAL_NODE_MAX_CELLS {
            panic!("Trying to insert cell into a full leaf node");
        }

        let mut cell = [0; INTERNAL_CELL_SIZE];
        cell[..ID_SIZE].copy_from_slice(&key.to_le_bytes());
        cell[ID_SIZE..].copy_from_slice(&value.to_le_bytes());
        self.keys.insert(index, InternalCell(cell));
        self.keys_count += 1;
    }

    pub fn internal_node_insert(&mut self, key: usize, value: usize) {
        if self.keys_count >= INTERNAL_NODE_MAX_CELLS {
            panic!("Trying to insert cell into a full leaf node");
        }

        let mut cell = [0; INTERNAL_CELL_SIZE];
        cell[..ID_SIZE].copy_from_slice(&key.to_le_bytes());
        cell[ID_SIZE..].copy_from_slice(&value.to_le_bytes());
        self.keys.insert(self.keys_count, InternalCell(cell));
        self.keys_count += 1;
    }

    pub fn get_parent_page_num(&self) -> usize {
        self.parent_page_num
    }

    pub fn get_mut_cell(&mut self, cell_index: usize) -> &mut Cell {
        &mut self.cells[cell_index]
    }

    pub fn remove_cell(&mut self, cell_index: usize) {
        self.cells.remove(cell_index);
        self.cells_count -= 1;
    }

    pub fn get_value(&self, cell_index: usize) -> &[u8] {
        &self.cells[cell_index].0[ID_SIZE..]
    }

    pub fn get_key(&self, cell_index: usize) -> usize {
        let key_bytes = &self.cells[cell_index].0[..ID_SIZE];
        usize::from_le_bytes(key_bytes.try_into().unwrap())
    }

    pub fn internal_get_key(&self, cell_index: usize) -> usize {
        let key_bytes = &self.keys[cell_index].0[..ID_SIZE];
        usize::from_le_bytes(key_bytes.try_into().unwrap())
    }

    pub fn internal_get_value(&self, cell_index: usize) -> usize {
        let key_bytes = &self.keys[cell_index].0[ID_SIZE..];
        usize::from_le_bytes(key_bytes.try_into().unwrap())
    }

    pub fn get_internal_key(&self, cell_index: usize) -> usize {
        let key_bytes = &self.keys[cell_index].0[..ID_SIZE];
        usize::from_le_bytes(key_bytes.try_into().unwrap())
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = vec![0; PAGE_SIZE];
        bytes[NODE_TYPE_OFFSET] = match self.node_type {
            NodeType::Leaf => 0,
            NodeType::Internal => 1,
        };
        bytes[IS_ROOT_OFFSET] = if self.is_root { 1 } else { 0 };
        bytes[PARENT_PAGE_NUM_OFFSET..PARENT_PAGE_NUM_OFFSET + PARENT_PAGE_NUM_SIZE]
            .copy_from_slice(&self.parent_page_num.to_le_bytes());

        bytes[CELLS_COUNT_OFFSET..CELLS_COUNT_OFFSET + CELLS_COUNT_SIZE]
            .copy_from_slice(&self.cells_count.to_le_bytes());

        let mut cells_offset = LEAF_NODE_CELLS_OFFSET;
        for cell in &self.cells {
            bytes[cells_offset..cells_offset + CELL_SIZE].copy_from_slice(&cell.0);
            cells_offset += CELL_SIZE;
        }

        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Node {
        let node_type = match bytes[NODE_TYPE_OFFSET] {
            0 => NodeType::Leaf,
            1 => NodeType::Internal,
            _ => panic!("Unknown node type {}", bytes[NODE_TYPE_OFFSET]),
        };
        return match node_type {
            NodeType::Leaf => {
                Node::deserialize_leaf_node(bytes)
            }
            NodeType::Internal => {
                Node::deserialize_internal_node(bytes)
            }
        };
    }

    fn deserialize_internal_node(bytes: &[u8]) -> Node {
        let is_root = bytes[IS_ROOT_OFFSET] == 1;
        let parent_page_num = usize::from_le_bytes(
            bytes[PARENT_PAGE_NUM_OFFSET..PARENT_PAGE_NUM_OFFSET + PARENT_PAGE_NUM_SIZE]
                .try_into()
                .unwrap(),
        );

        let keys_count = usize::from_le_bytes(
            bytes[CELLS_COUNT_OFFSET..CELLS_COUNT_OFFSET + CELLS_COUNT_SIZE]
                .try_into()
                .unwrap(),
        );

        let mut keys = Vec::new();
        let mut keys_offset = KEY_VALUE_OFFSET;
        for _ in 0..keys_count {
            let mut key = [0; KEY_VALUE_SIZE];
            key.copy_from_slice(&bytes[keys_offset..keys_offset + INTERNAL_CELL_SIZE]);
            keys.push(InternalCell(key));
            keys_offset += INTERNAL_CELL_SIZE;
        }

        let right_child_key = usize::from_le_bytes(
            bytes[RIGHT_CHILD_OFFSET..RIGHT_CHILD_OFFSET + PAGE_NUM_SIZE]
                .try_into()
                .unwrap(),
        );

        Node {
            node_type: NodeType::Internal,
            is_root,
            parent_page_num,
            cells: Vec::new(),
            cells_count: 0,
            keys,
            keys_count,
            right_child_key,
            next_leaf_num: 0,
        }
    }

    fn deserialize_leaf_node(bytes: &[u8]) -> Node {
        let is_root = bytes[IS_ROOT_OFFSET] == 1;
        let parent_page_num = usize::from_le_bytes(
            bytes[PARENT_PAGE_NUM_OFFSET..PARENT_PAGE_NUM_OFFSET + PARENT_PAGE_NUM_SIZE]
                .try_into()
                .unwrap(),
        );

        let cells_count = usize::from_le_bytes(
            bytes[CELLS_COUNT_OFFSET..CELLS_COUNT_OFFSET + CELLS_COUNT_SIZE]
                .try_into()
                .unwrap(),
        );

        let mut cells = Vec::new();
        let mut cells_offset = LEAF_NODE_CELLS_OFFSET;
        for _ in 0..cells_count {
            let mut cell = [0; CELL_SIZE];
            cell.copy_from_slice(&bytes[cells_offset..cells_offset + CELL_SIZE]);
            cells.push(Cell(cell));
            cells_offset += CELL_SIZE;
        }

        Node {
            node_type: NodeType::Leaf,
            is_root,
            parent_page_num,
            cells,
            cells_count,
            keys_count: 0,
            keys: Vec::new(),
            right_child_key: 0,
            next_leaf_num: 0,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct Cell([u8; CELL_SIZE]);

#[derive(Clone, PartialEq, Debug)]
pub struct InternalCell([u8; INTERNAL_CELL_SIZE]);

#[derive(Debug, PartialEq, Clone)]
pub enum NodeType {
    Leaf,
    Internal,
}

#[cfg(test)]
mod test {
    use crate::storage::row::Row;

    use super::*;

    #[test]
    fn test_node() {
        let mut node = Node::new_leaf();
        let key: usize = 1;
        let value = Row::new(1, "test".to_string(), "test".to_string()).serialize().unwrap();
        node.insert_key_value(key, &value, 0);
        let serialized = node.serialize();
        let deserialized = Node::deserialize(&serialized);
        assert_eq!(deserialized.node_type, NodeType::Leaf);
        assert_eq!(deserialized.is_root, false);
        assert_eq!(deserialized.parent_page_num, 0);
        assert_eq!(deserialized.cells_count, 1);
    }

    #[test]
    fn test_node_insert() {
        let mut node = Node::new_leaf();
        let key: usize = 1;
        let value = Row::new(1, "test".to_string(), "test".to_string()).serialize().unwrap();
        node.insert_key_value(key, &value, 0);
        assert_eq!(node.get_cell_count(), 1);
        assert_eq!(node.get_key(0), key);
        assert_eq!(node.get_value(0), value);
    }

    #[test]
    fn test_node_deserialize() {
        let mut node = Node::new_leaf();
        let key: usize = 1;
        let value = Row::new(1, "test".to_string(), "test".to_string()).serialize().unwrap();
        node.insert_key_value(key, &value, 0);
        let serialized = node.serialize();
        let deserialized = Node::deserialize(&serialized);
        assert_eq!(deserialized.node_type, NodeType::Leaf);
        assert_eq!(deserialized.is_root, false);
        assert_eq!(deserialized.parent_page_num, 0);
        assert_eq!(deserialized.cells_count, 1);
    }

    #[test]
    fn test_node_internal() {
        let mut node = Node::new_internal();
        let key: usize = 1;
        let value = Row::new(1, "test".to_string(), "test".to_string()).serialize().unwrap();
        node.insert_key_value(key, &value, 0);
        let serialized = node.serialize();
        let deserialized = Node::deserialize(&serialized);
        assert_eq!(deserialized.node_type, NodeType::Internal);
        assert_eq!(deserialized.is_root, false);
        assert_eq!(deserialized.parent_page_num, 0);
        assert_eq!(deserialized.keys_count, 1);
    }
}

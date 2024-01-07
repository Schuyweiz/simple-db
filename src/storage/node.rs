use std::usize;

use crate::storage::constant::{ID_SIZE, PAGE_SIZE};
// unused imports will be kept until the end of the project to know if they are really unused
use crate::storage::constant::{
    CELL_SIZE, CELLS_COUNT_OFFSET, CELLS_COUNT_SIZE, IS_ROOT_OFFSET,
    LEAF_NODE_CELLS_OFFSET, LEAF_NODE_MAX_CELLS,
    NODE_TYPE_OFFSET, PARENT_PAGE_NUM_OFFSET, PARENT_PAGE_NUM_SIZE
    ,
};

// replaced Page from previous implementation. Page structure will be restored later on if deemed necessary
#[derive(Clone)]
pub struct Node {
    // meta, common
    //todo: should be a separate struct perhaps?
    node_type: NodeType,
    is_root: bool,
    parent_page_num: usize,

    //meta leaf node
    //need to be here for manual deserialization without billion of rows with 0 values
    cells_count: usize,
    cells: Vec<Cell>,
}

impl Node {
    pub fn new_leaf() -> Node {
        Node {
            node_type: NodeType::Leaf,
            is_root: false,
            parent_page_num: 0,
            cells_count: 0,
            cells: Vec::new(),
        }
    }

    fn insert_cell(&mut self, cell: Cell) {
        if self.cells_count >= LEAF_NODE_MAX_CELLS {
            panic!("Trying to insert cell into a full leaf node");
        }

        // i really dont like this, but file deser requires cells_count to work
        // need a better way to serialzie cells to solve this one.
        self.cells.push(cell);
        self.cells_count += 1;
    }

    pub fn insert_key_value(&mut self, key: &[u8], value: &[u8]) {
        let mut cell = [0; CELL_SIZE];
        cell[..ID_SIZE].copy_from_slice(key);
        cell[ID_SIZE..].copy_from_slice(value);
        self.insert_cell(Cell(cell));
    }

    pub fn get_cell_count(&self) -> usize {
        self.cells_count
    }

    pub fn get_parent_page_num(&self) -> usize {
        self.parent_page_num
    }

    pub fn get_mut_cell(&mut self, cell_index: usize) -> &mut Cell {
        &mut self.cells[cell_index]
    }

    pub fn get_value(&self, cell_index: usize) -> &[u8] {
        &self.cells[cell_index].0[ID_SIZE..]
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
            node_type,
            is_root,
            parent_page_num,
            cells,
            cells_count,
        }
    }
}

#[derive(Clone)]
pub(crate) struct Cell([u8; CELL_SIZE]);

#[derive(Debug, PartialEq, Clone)]
enum NodeType {
    Leaf,
    Internal,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_leaf_node() {
        let node = Node::new_leaf();
        assert_eq!(node.node_type, NodeType::Leaf);
        assert_eq!(node.is_root, false);
        assert_eq!(node.parent_page_num, 0);
        assert_eq!(node.cells.len(), 0);
    }

    #[test]
    fn serialize_deserialize_leaf_node() {
        let mut node = Node::new_leaf();
        let cell = Cell([1; CELL_SIZE]);
        node.insert_cell(cell);
        let bytes = node.serialize();
        let deserialized_node = Node::deserialize(&bytes);
        assert_eq!(deserialized_node.node_type, NodeType::Leaf);
        assert_eq!(deserialized_node.is_root, false);
        assert_eq!(deserialized_node.parent_page_num, 0);
        assert_eq!(deserialized_node.cells.len(), 1);
        assert_eq!(deserialized_node.cells[0].0, [1; CELL_SIZE]);
    }

    #[test]
    fn serialize_deserialize_leaf_node_with_multiple_cells() {
        let mut node = Node::new_leaf();
        let cell = Cell([1; CELL_SIZE]);
        node.insert_cell(cell);
        let cell = Cell([2; CELL_SIZE]);
        node.insert_cell(cell);
        let bytes = node.serialize();
        let deserialized_node = Node::deserialize(&bytes);
        assert_eq!(deserialized_node.node_type, NodeType::Leaf);
        assert_eq!(deserialized_node.is_root, false);
        assert_eq!(deserialized_node.parent_page_num, 0);
        assert_eq!(deserialized_node.cells.len(), 2);
        assert_eq!(deserialized_node.cells[0].0, [1; CELL_SIZE]);
        assert_eq!(deserialized_node.cells[1].0, [2; CELL_SIZE]);
    }
}

use crate::storage::constant::{ID_SIZE, PAGE_SIZE, ROW_SIZE};

const NODE_TYPE_SIZE: usize = std::mem::size_of::<u8>();
const NODE_TYPE_OFFSET: usize = 0;

const IS_ROOT_SIZE: usize = std::mem::size_of::<u8>();
const IS_ROOT_OFFSET: usize = NODE_TYPE_OFFSET + NODE_TYPE_SIZE;

const PARENT_PAGE_NUM_SIZE: usize = std::mem::size_of::<i32>();
const PARENT_PAGE_NUM_OFFSET: usize = IS_ROOT_OFFSET + IS_ROOT_SIZE;

const SPACE_FOR_COMMON_HEADER: usize = NODE_TYPE_SIZE + IS_ROOT_SIZE + PARENT_PAGE_NUM_SIZE;

const CELLS_COUNT_SIZE: usize = std::mem::size_of::<usize>();
const CELLS_COUNT_OFFSET: usize = SPACE_FOR_COMMON_HEADER;
const LEAF_NODE_HEADER_SIZE: usize = CELLS_COUNT_SIZE;

const LEAF_NODE_CELLS_SPACE: usize = PAGE_SIZE - SPACE_FOR_COMMON_HEADER - LEAF_NODE_HEADER_SIZE;
const LEAF_NODE_CELLS_OFFSET: usize = SPACE_FOR_COMMON_HEADER + LEAF_NODE_HEADER_SIZE;

const LEAF_NODE_MAX_CELLS: usize = LEAF_NODE_CELLS_SPACE / std::mem::size_of::<Cell>();

const CELL_SIZE: usize = ID_SIZE + ROW_SIZE;

pub struct Node {
    // meta, common
    //todo: should be a separate struct perhaps?
    node_type: NodeType,
    is_root: bool,
    parent_page_num: i32,

    //meta leaf node
    //need to be here for manual deserialization without bajillion of rows with 0 values
    cells_count: usize,
    cells: Vec<Cell>,
}

impl Node {
    pub fn new_leaf() -> Node {
        Node {
            node_type: NodeType::Leaf,
            is_root: false,
            parent_page_num: -1,
            cells_count: 0,
            cells: Vec::new(),
        }
    }

    pub fn insert_cell(&mut self, cell: Cell) {
        self.cells.push(cell);
        self.cells_count += 1;
    }

    pub fn get_parent_page_num(&self) -> i32 {
        self.parent_page_num
    }

    pub fn get_mut_cell(&mut self, cell_index: usize) -> &mut Cell {
        &mut self.cells[cell_index]
    }

    pub fn get_cell(&self, cell_index: usize) -> &Cell {
        &self.cells[cell_index]
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
        let parent_page_num = i32::from_le_bytes(
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

struct Cell([u8; CELL_SIZE]);

#[derive(Debug, PartialEq)]
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
        assert_eq!(node.parent_page_num, -1);
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
        assert_eq!(deserialized_node.parent_page_num, -1);
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
        assert_eq!(deserialized_node.parent_page_num, -1);
        assert_eq!(deserialized_node.cells.len(), 2);
        assert_eq!(deserialized_node.cells[0].0, [1; CELL_SIZE]);
        assert_eq!(deserialized_node.cells[1].0, [2; CELL_SIZE]);
    }
}
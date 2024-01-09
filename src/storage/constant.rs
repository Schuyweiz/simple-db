// row constants
pub const USER_NAME_SIZE: usize = 32;
pub const EMAIL_SIZE: usize = 255;
pub const ID_SIZE: usize = std::mem::size_of::<usize>();
pub const ROW_SIZE: usize = USER_NAME_SIZE + EMAIL_SIZE + ID_SIZE;
// table constants
pub const PAGE_SIZE: usize = 4096;

pub const TABLE_MAX_PAGES: usize = 100;

// node constants

pub const NODE_TYPE_SIZE: usize = std::mem::size_of::<u8>();
pub(crate) const NODE_TYPE_OFFSET: usize = 0;

pub(crate) const IS_ROOT_SIZE: usize = std::mem::size_of::<u8>();
pub(crate) const IS_ROOT_OFFSET: usize = NODE_TYPE_OFFSET + NODE_TYPE_SIZE;

pub(crate) const PARENT_PAGE_NUM_SIZE: usize = std::mem::size_of::<usize>();
pub(crate) const PARENT_PAGE_NUM_OFFSET: usize = IS_ROOT_OFFSET + IS_ROOT_SIZE;

pub(crate) const SPACE_FOR_COMMON_HEADER: usize = NODE_TYPE_SIZE + IS_ROOT_SIZE + PARENT_PAGE_NUM_SIZE;

// leaf node constants
pub(crate) const CELLS_COUNT_SIZE: usize = std::mem::size_of::<usize>();
pub(crate) const CELLS_COUNT_OFFSET: usize = SPACE_FOR_COMMON_HEADER;
pub const LEAF_NEXT_LEAF_SIZE: usize = std::mem::size_of::<usize>();
pub const LEAF_NEXT_LEAF_OFFSET: usize = CELLS_COUNT_OFFSET + CELLS_COUNT_SIZE;

pub(crate) const LEAF_NODE_HEADER_SIZE: usize = CELLS_COUNT_SIZE + LEAF_NEXT_LEAF_SIZE;

pub(crate) const LEAF_NODE_CELLS_SPACE: usize = PAGE_SIZE - SPACE_FOR_COMMON_HEADER - LEAF_NODE_HEADER_SIZE;
pub(crate) const LEAF_NODE_CELLS_OFFSET: usize = SPACE_FOR_COMMON_HEADER + LEAF_NODE_HEADER_SIZE;

pub(crate) const LEAF_NODE_MAX_CELLS: usize = 3;

pub(crate) const CELL_SIZE: usize = ID_SIZE + ROW_SIZE;

// internal node constants
pub const INTERNAL_NODE_KEY_COUNT_SIZE: usize = std::mem::size_of::<usize>();
pub const INTERNAL_NODE_KEY_COUNT_OFFSET: usize = SPACE_FOR_COMMON_HEADER;
pub const PAGE_NUM_SIZE: usize = std::mem::size_of::<usize>();
pub const RIGHT_CHILD_OFFSET: usize = INTERNAL_NODE_KEY_COUNT_OFFSET + INTERNAL_NODE_KEY_COUNT_SIZE;
pub const INTERNAL_NODE_HEADER_SIZE: usize = INTERNAL_NODE_KEY_COUNT_SIZE + PAGE_NUM_SIZE;

pub const KEY_VALUE_SIZE: usize = ID_SIZE + PAGE_NUM_SIZE;
pub const KEY_VALUE_OFFSET: usize = SPACE_FOR_COMMON_HEADER + INTERNAL_NODE_HEADER_SIZE;
pub const INTERNAL_NODE_MAX_CELLS: usize = 3;
pub const INTERNAL_CELL_SIZE: usize = ID_SIZE + PAGE_NUM_SIZE;

// row constants
pub const USER_NAME_SIZE: usize = 32;
pub const EMAIL_SIZE: usize = 255;
pub const ID_SIZE: usize = std::mem::size_of::<u32>();
pub const ROW_SIZE: usize = USER_NAME_SIZE + EMAIL_SIZE + ID_SIZE;
// table constants
pub const PAGE_SIZE: usize = 4096;

pub const TABLE_MAX_PAGES: usize = 100;
// should be represented as row size / the size of a generic type in the table
pub const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;

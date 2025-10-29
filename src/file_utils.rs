use nfscrs::OpenOptions;

pub const READ_BIT_NUM: usize = 0;
pub const WRITE_BIT_NUM: usize = 1;
pub const CREATE_BIT_NUM: usize = 2;
pub const TRUNCATE_BIT_NUM: usize = 3;

pub fn int_to_open_options(i: i32) -> OpenOptions {
    OpenOptions::new()
        .read((i & 1 << READ_BIT_NUM) != 0)
        .write((i & 1 << WRITE_BIT_NUM) != 0)
        .create((i & 1 << CREATE_BIT_NUM) != 0)
        .truncate((i & 1 << TRUNCATE_BIT_NUM) != 0)
}

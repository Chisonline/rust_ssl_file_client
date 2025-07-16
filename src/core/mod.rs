pub mod req;
pub mod client;
pub mod biz;

#[allow(unused)]
pub const MAX_BLOCK_SIZE: usize = 16 * MB;

pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;
pub const GB: usize = 1024 * MB;
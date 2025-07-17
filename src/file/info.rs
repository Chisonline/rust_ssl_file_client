use crate::{control::ControlBlock, core::biz::{self, ListFileResp}};

pub async fn list_file(filter: String) -> Result<ListFileResp, Box<dyn std::error::Error>> {
    biz::list_file(filter).await
}

pub async fn delete_file(block: ControlBlock, file_id: i32) -> Result<(), Box<dyn std::error::Error>> {
    biz::delete_file(block, file_id).await
}
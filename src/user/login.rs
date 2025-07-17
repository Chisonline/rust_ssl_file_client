use crate::{control::ControlBlock, core::biz};

pub async fn login(block: &mut ControlBlock, user_name: String, password: String) -> Result<(), Box<dyn std::error::Error>> {
    biz::login(block, user_name, password).await
}

pub async fn register(block: &mut ControlBlock, user_name: String, password: String) -> Result<(), Box<dyn std::error::Error>> {
    biz::register(block, user_name, password).await
}
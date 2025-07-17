use crate::{control::ControlBlock, core::biz};

pub async fn refresh(block: &mut ControlBlock) -> Result<(), Box<dyn std::error::Error>> {
    let exp = block.exp;
    let now = chrono::Utc::now().timestamp();
    let threshold = chrono::Duration::seconds(60 * 60 * 12);

    if exp - now < threshold.num_seconds() {
        biz::refresh(block).await
    } else {
        Ok(())
    }
}

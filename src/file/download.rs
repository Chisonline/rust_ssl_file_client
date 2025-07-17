use std::sync::Arc;

use tokio::{io::AsyncWriteExt as _, sync::Semaphore};
use uuid::Uuid;

use crate::{control::ControlBlock, core::biz};

fn make_prefix(file_id: i32) -> String {
    let uuid = Uuid::new_v4();
    format!("{}_{}", file_id, uuid)
}

pub async fn download(
    block: ControlBlock,
    file_id: i32,
    target_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let block_ids = biz::get_block_ids(block.clone(), file_id).await?.block_ids;

    let semaphore = Arc::new(Semaphore::new(16));

    let prefix = make_prefix(file_id);

    let handles = block_ids
        .iter()
        .map(|block_id| {
            let semaphore = semaphore.clone();
            let block = block.clone();
            let block_id = *block_id;
            let prefix = prefix.clone();
            let target_path = target_path.to_owned();

            tokio::task::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                for _ in 0..3 {
                    let block_use = block.clone();
                    let rst = biz::get_block(block_use, block_id).await;
                    if let Some(resp) = rst {
                        let block_info = resp.block_info;
                        let block_data = resp.block_data;
                        let name = format!("{}_{}", prefix, block_info.id);
                        let path = format!("{}/{name}", target_path.clone());

                        if let Ok(_) = tokio::fs::write(&path, block_data).await {
                            break
                        }
                    }
                }
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        handle.await?;
    }

    let block_vec = search_files_by_prefix(target_path, prefix.as_str()).await?;

    join_files(block_vec, target_path, &format!("{}.bin", file_id)).await?;

    Ok(())
}

async fn search_files_by_prefix(dir: &str, prefix: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    let mut dir_entries = tokio::fs::read_dir(dir).await?;
    while let Some(entry) = dir_entries.next_entry().await? {
        let path = entry.path();

        if path.is_file() {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            if file_name.starts_with(prefix) {
                files.push(path.to_string_lossy().to_string());
            }
        }
    }

    files.sort();

    Ok(files)
}

async fn join_files(block_vec: Vec<String>, target_path: &str, target_file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = tokio::fs::File::create(format!("{}/{}", target_path, target_file_name)).await?;

    for block in block_vec {
        let block_data = tokio::fs::read(format!("{}/{block}", target_path)).await?;
        file.write_all(&block_data).await?;
    }

    Ok(())
}
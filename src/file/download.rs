use std::sync::{Arc, Mutex};
use crc_fast::{checksum,  checksum_file, CrcAlgorithm::Crc32IsoHdlc};
use tokio::{io::AsyncWriteExt as _, sync::Semaphore};
use uuid::Uuid;

use crate::{control::ControlBlock, core::{biz, req::async_debug}};

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

    let mutex_flag = Arc::new(Mutex::new(true));

    let handles = block_ids
        .iter()
        .map(|block_id| {
            let semaphore = semaphore.clone();
            let block = block.clone();
            let block_id = *block_id;
            let prefix = prefix.clone();
            let target_path = target_path.to_owned();

            let mutex_flag = mutex_flag.clone();

            tokio::task::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let mut success = false;
                for _ in 0..3 {
                    if !*mutex_flag.lock().unwrap() {
                        break;
                    }

                    let block_use = block.clone();
                    let rst = biz::get_block(block_use, block_id).await;
                    if let Some(resp) = rst {
                        let block_info = resp.block_info;
                        let block_data = resp.block_data;

                        let block_checksum = block_info.block_checksum;
                        if block_checksum != checksum(Crc32IsoHdlc, &block_data) as u32 {
                            continue;
                        }

                        let name = format!("{}_{}", prefix, block_info.block_id);
                        let path = format!("{}/{name}", target_path.clone());

                        let mut file = match tokio::fs::File::create(path).await {
                            Ok(file) => file,
                            Err(_) => continue,
                        };

                        match file.write_all(&block_data).await {
                            Ok(_) => {
                                success = true;
                                break;
                            },
                            Err(_) => continue,
                        };
                    }
                }

                if !success {
                    *mutex_flag.lock().unwrap() = false;
                }
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        handle.await?;
    }

    let flag = *mutex_flag.lock().unwrap();
    async_debug(format!("mutex_flag: {flag}")).await;

    if !flag {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Download failed")));
    }

    let file_info = biz::get_file_info(file_id).await?;
    async_debug(format!("{:?}", file_info)).await;
    let file_name = file_info.file_name;
    let file_checksum = file_info.file_checksum;

    let block_vec = search_files_by_prefix(target_path, prefix.as_str()).await?;

    async_debug(format!("{:?}", block_vec)).await;
    join_files(block_vec.clone(), target_path, &format!("{}", file_name)).await?;

    check_file(target_path, &format!("{}", file_name), file_checksum).await?;

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

    files.sort_by_key(|file| {
        let parts: Vec<&str> = file.split('_').collect();
        if let Some(last_part) = parts.last() {
            if let Ok(num) = last_part.parse::<u32>() {
                return num;
            }
        }
        u32::MAX
    });

    Ok(files)
}

async fn join_files(block_vec: Vec<String>, target_path: &str, target_file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = tokio::fs::File::create(format!("{}/{}", target_path, target_file_name)).await?;

    for block in &block_vec {
        let block_data = tokio::fs::read(block).await?;
        file.write_all(&block_data).await?;
    }

    for block in block_vec {
        tokio::fs::remove_file(block).await?;
    }

    Ok(())
}

async fn check_file(target_path: &str, target_file_name: &str, file_checksum: u32) -> Result<(), Box<dyn std::error::Error>> {
    
    let crc32 = checksum_file(Crc32IsoHdlc, &format!("{}/{}", target_path, target_file_name), None)?;

    if crc32 as u32 != file_checksum {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("check_file failed, {} vs {}", crc32, file_checksum))));
    }

    Ok(())
}

#[tokio::test]
async fn test_search_by_prefix() {
    let dir = "./download";
    let prefix = "26_";
    let rst = search_files_by_prefix(dir, prefix).await.unwrap();
    for strr in rst {
        println!("{}", strr)
    }
}
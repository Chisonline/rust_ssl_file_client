use std::sync::Arc;

use crc_fast::{checksum, checksum_combine, CrcAlgorithm::Crc32IsoHdlc};
use serde::{Deserialize, Serialize};
use tokio::{io::{self, AsyncReadExt, AsyncSeekExt}, sync::{mpsc::Permit, Semaphore}};

use crate::{control::ControlBlock, core::{req::{req_server, Payload, Resp}, GB, KB, MB}, core::biz};

pub async fn upload(block: ControlBlock, file_name: &str, path: String) -> Result<(), Box<dyn std::error::Error>> {
    let metadata = tokio::fs::metadata(&path).await?;
    let file_size = metadata.len() as usize;
    let granularity = calcu_granularity(file_size);

    let semaphore = Arc::new(Semaphore::new(8));
    let mut handles = Vec::new();
    let file = tokio::fs::File::open(path).await?;
    let mut buffer = Vec::with_capacity(granularity);
    let mut position = 0;

    let block_clone = block.clone();
    let file_id = biz::presend(block_clone, file_name, file_size).await?;

    let mut block_id = 0;
    let mut crc32 = 0;

    loop {
        let mut file_clone = file.try_clone().await?;
        file_clone.seek(io::SeekFrom::Start(position)).await?;

        buffer.clear();

        let bytes_read = file_clone.take(granularity as u64).read_to_end(&mut buffer).await?;

        if bytes_read == 0 {
            break;
        }

        position += bytes_read as u64;
        let chunk_data = String::from_utf8_lossy(&buffer).into_owned();

        let block_checksum = checksum(Crc32IsoHdlc, chunk_data.as_bytes());
        crc32 = checksum_combine(Crc32IsoHdlc, crc32, block_checksum, bytes_read as u64);

        let semaphore_clone = Arc::clone(&semaphore);
        let block_clone = block.clone();
        
        let handle = tokio::task::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();
            for _ in 0..3 {
                let block_use = block_clone.clone();
                let data_use = chunk_data.clone();
                let rst = biz::send(block_use, file_id, block_id, block_checksum as u32, data_use).await;
                if let Ok(_) = rst {
                    break;
                }
            }
        });

        block_id += 1;
        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    biz::finish(block, file_id, crc32 as u32).await
}

fn calcu_granularity(size: usize) -> usize {
    if size < 16 * MB {
        return 128 * KB;
    }
    if size < 64 * MB {
        return 512 * KB;
    }
    if size < 128 * MB {
        return 2 * MB;
    }
    if size < 1 * GB {
        return 8 * MB;
    }
    return 16 * MB;
}

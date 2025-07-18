use std::sync::Arc;

use crc_fast::{checksum, checksum_file, CrcAlgorithm::Crc32IsoHdlc};
use tokio::{
    io::{self, AsyncReadExt, AsyncSeekExt},
    sync::{Mutex, Semaphore},
};

use crate::{
    control::ControlBlock,
    core::biz,
    core::{GB, KB, MB},
};

pub async fn upload(
    block: ControlBlock,
    file_name: &str,
    path: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let metadata = tokio::fs::metadata(format!("{}/{}", path, file_name)).await?;
    let file_size = metadata.len() as usize;
    let granularity = calcu_granularity(file_size);

    let semaphore = Arc::new(Semaphore::new(8));
    let mut handles = Vec::new();
    let file = tokio::fs::File::open(format!("{}/{}", path, file_name)).await?;
    let mut buffer = Vec::with_capacity(granularity);
    let mut position = 0;

    let block_clone = block.clone();
    let file_id = biz::presend(block_clone, file_name, file_size).await?;

    let mut block_id = 0;

    let mutex_flag = Arc::new(Mutex::new(true));

    loop {
        let mut file_clone = file.try_clone().await?;
        file_clone.seek(io::SeekFrom::Start(position)).await?;

        buffer.clear();

        let bytes_read = file_clone
            .take(granularity as u64)
            .read_to_end(&mut buffer)
            .await?;

        if bytes_read == 0 {
            break;
        }

        position += bytes_read as u64;

        let block_checksum = checksum(Crc32IsoHdlc, &buffer);

        let semaphore_clone = Arc::clone(&semaphore);
        let block_clone = block.clone();

        let mutex_flag = mutex_flag.clone();
        let data_use =  buffer.clone();
        
        let handle = tokio::task::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();
            let mut success = false;
            for _ in 0..3 {
                if !*mutex_flag.lock().await {
                    break;
                }

                let block_use = block_clone.clone();
                let data_use =  data_use.clone();
                let rst = biz::send(
                    block_use,
                    file_id,
                    block_id,
                    block_checksum as u32,
                    data_use,
                )
                .await;
                if let Ok(_) = rst {
                    success = true;
                    break;
                }
            }
            if !success {
                *mutex_flag.lock().await = false;
            }
        });

        block_id += 1;
        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    let flag = *mutex_flag.lock().await;
    if !flag {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Upload failed",
        )));
    }

    let crc32 = checksum_file(Crc32IsoHdlc, &format!("{}/{}", path, file_name), None)?;

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

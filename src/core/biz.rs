use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{control::ControlBlock, core::req::{req_server, Payload, Resp}};

#[derive(Serialize, Debug)]
struct PresendReq {
    pub file_name: String,
    pub file_size: u64,
}

pub async fn presend(block: ControlBlock, file_name: &str, file_size: usize) -> Result<u32, Box<dyn std::error::Error>> {
    let req = PresendReq {
        file_name: file_name.to_string(),
        file_size: file_size as u64,
    };

    let payload = Payload {
        method: "presend".to_string(),
        block: Some(block),
        content: Some(req),
    };

    let resp: Resp<u32> = req_server(payload).await?;

    if !resp.success {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "presend failed")));
    }

    let file_id = match resp.content {
        Some(id) => id,
        None => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "presend failed"))),
    };

    Ok(file_id)
}

#[derive(Serialize, Debug)]
struct SendReq {
    pub file_id: u32,
    pub block_id: u64,
    pub block_checksum: u32,
    pub block_payload: Vec<u8>,
}

pub async fn send(block: ControlBlock, file_id: u32, block_id: u64, block_checksum: u32, block_payload: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let req = SendReq {
        file_id,
        block_id,
        block_checksum,
        block_payload,
    };

    let payload = Payload {
        method: "send".to_string(),
        block: Some(block),
        content: Some(req),
    };

    let resp: Resp<()> = req_server(payload).await?;

    if !resp.success {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "send failed")));
    }

    Ok(())
}

#[derive(Serialize, Debug)]
struct FinishReq {
    pub file_id: u32,
    pub file_checksum: u32,
}

pub async fn finish(block: ControlBlock, file_id: u32, file_checksum: u32) -> Result<(), Box<dyn std::error::Error>> {
    let req = FinishReq {
        file_id,
        file_checksum,
    };

    let payload = Payload {
        method: "finish".to_string(),
        block: Some(block),
        content: Some(req),
    };

    let resp: Resp<()> = req_server(payload).await?;

    if !resp.success {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "finish failed")));
    }

    Ok(())
}

#[derive(Serialize, Debug)]
pub struct GetBlockIdsByFileIdReq {
    file_id: i32,
}

#[derive(Deserialize, Debug)]
pub struct GetBlockIdsByFileIdResp {
    pub block_ids: Vec<i32>,
}

pub async fn get_block_ids(block: ControlBlock, file_id: i32) -> Result<GetBlockIdsByFileIdResp, Box<dyn std::error::Error>> {
    let req = GetBlockIdsByFileIdReq {
        file_id,
    };

    let payload = Payload {
        method: "get_block_ids".to_string(),
        block: Some(block),
        content: Some(req),
    };

    let resp: Resp<GetBlockIdsByFileIdResp> = req_server(payload).await?;

    if !resp.success {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "get_block_ids failed")));
    }

    match resp.content {
        Some(block_ids) => Ok(block_ids),
        None => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "get_block_ids failed"))),
    }
}

#[derive(Serialize, Debug)]
pub struct GetBlockReq {
    block_id: i32,
}

#[derive(Deserialize, Debug)]
pub struct GetBlockResp {
    pub block_info: FileBlock,
    pub block_data: Vec<u8>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileBlock {
    pub id: i32,
    pub file_id: i32,
    pub block_name: String,
    pub block_id: i64,
    pub block_checksum: u32,
    pub block_size: u32,
    pub created_at: NaiveDateTime,
}

pub async fn get_block(block: ControlBlock, block_id: i32) -> Option<GetBlockResp> {
    let req = GetBlockReq {
        block_id,
    };

    let payload = Payload {
        method: "get_block".to_string(),
        block: Some(block),
        content: Some(req),
    };

    let resp: Resp<GetBlockResp> = match req_server(payload).await {
        Ok(resp) => resp,
        Err(_) => return None,
    };

    if !resp.success {
        return None;
    }

    resp.content
}

#[derive(Serialize, Debug)]
pub struct ListFileReq {
    filter: String
}

#[derive(Deserialize, Debug)]
pub struct ListFileResp {
    pub file_info: Vec<FileInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    pub id: i32,
    pub file_name: String,
    pub file_size: i64,
    pub file_checksum: u32,
    pub file_status: i32,
    pub created_at: NaiveDateTime,
}

pub async fn list_file(filter: String) -> Result<ListFileResp, Box<dyn std::error::Error>> {
    let req = ListFileReq {
        filter,
    };

    let payload = Payload {
        method: "list_file".to_string(),
        block: None,
        content: Some(req),
    };

    let resp: Resp<ListFileResp> = req_server(payload).await?;

    if !resp.success {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "list_file failed")));
    }

    match resp.content {
        Some(file_info) => Ok(file_info),
        None => Ok(ListFileResp { file_info: Vec::new() }),
    }
}

#[derive(Serialize, Debug)]
pub struct DeleteFileReq {
    file_id: i32,
}

pub async fn delete_file(block: ControlBlock, file_id: i32) -> Result<(), Box<dyn std::error::Error>> {
    let req = DeleteFileReq {
        file_id,
    };

    let payload = Payload {
        method: "delete_file".to_string(),
        block: Some(block),
        content: Some(req),
    };

    let resp: Resp<()> = req_server(payload).await?;

    if !resp.success {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "delete_file failed")));
    }

    Ok(())
}

#[allow(unused)]
pub async fn ping() -> Result<(), Box<dyn std::error::Error>> {
    let payload: Payload<u32> = Payload {
        method: "ping".to_string(),
        block: None,
        content: None,
    };

    let resp: Resp<()> = req_server(payload).await?;

    if !resp.success {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "ping failed")));
    }

    Ok(())
}

#[derive(Serialize, Debug)]
pub struct RegisterReq {
    pub user_name: String,
    pub password: String,
}

pub async fn register(block: &mut ControlBlock, user_name: String, password: String) -> Result<(), Box<dyn std::error::Error>> {
    let req = RegisterReq {
        user_name,
        password,
    };

    let block_clone = block.clone();

    let payload = Payload {
        method: "register".to_string(),
        block: Some(block_clone),
        content: Some(req),
    };

    let resp: Resp<()> = req_server(payload).await?;

    if !resp.success {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "register failed")));
    }

    *block = match resp.block {
        Some(block) => block,
        None => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "register failed"))),
    };

    Ok(())
}

#[derive(Serialize, Debug)]
pub struct LoginReq {
    pub user_name: String,
    pub password: String,
}

pub async fn login(block: &mut ControlBlock, user_name: String, password: String) -> Result<(), Box<dyn std::error::Error>> {
    let req = LoginReq {
        user_name,
        password,
    };

    let block_clone = block.clone();

    let payload = Payload {
        method: "login".to_string(),
        block: Some(block_clone),
        content: Some(req),
    };

    let resp: Resp<()> = req_server(payload).await?;

    if !resp.success {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "login failed")));
    }

    *block = match resp.block {
        Some(block) => block,
        None => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "login failed"))),
    };

    Ok(())
}

pub async fn refresh(block: &mut ControlBlock) -> Result<(), Box<dyn std::error::Error>> {
    let payload: Payload<u32> = Payload {
        method: "refresh".to_string(),
        block: Some(block.clone()),
        content: None,
    };

    let resp: Resp<()> = req_server(payload).await?;

    if !resp.success {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "refresh failed")));
    }

    *block = match resp.block {
        Some(block) => block,
        None => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "refresh failed"))),
    };

    Ok(())
}

#[derive(Serialize, Debug)]
pub struct GetFileInfoReq {
    file_id: i32,
}

pub async fn get_file_info(file_id: i32) -> Result<FileInfo, Box<dyn std::error::Error>> {
    let req = GetFileInfoReq {
        file_id: file_id
    };
    
    let payload= Payload {
        method: "get_file_info".to_string(),
        block: None,
        content: Some(req),
    };

    let resp: Resp<FileInfo> = req_server(payload).await?;

    if !resp.success {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "get_file_info failed")));
    }

    match resp.content {
        Some(file_info) => Ok(file_info),
        None => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "get_file_info failed"))),
    }
}
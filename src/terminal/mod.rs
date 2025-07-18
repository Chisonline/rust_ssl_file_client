use std::process::exit;

use dashmap::DashMap;
use tokio::{io::AsyncWriteExt, sync::OnceCell};
use crate::{control::ControlBlock, user::authorization::refresh};
use handler::*;
use tokio::io::{AsyncBufReadExt, BufReader};

mod handler;

pub async fn terminal() -> ! {
    let mut block = ControlBlock::default();
    let mut user: Option<String> = None;

    loop {

        if let Some(_) = user {
            if let Err(e) = refresh(&mut block).await {
                panic!("refresh token failed: {:?}", e);
            }
        }

        let (cmd, args) = input(user.clone()).await;

        match cmd.as_str() {
            "help" => help(args).await,
            "exit" => exit(0),
            "login" => {
                user = login(&mut block, args).await;
            },
            "register" => {
                user = register(&mut block, args).await;
            },
            "delete" => delete(block.clone(), args).await,
            "download" => download(block.clone(), args).await,
            "upload" => upload(block.clone(), args).await,
            "list_file" => list_file(args).await,
            "" => continue,
            _ => println!("unknown command: {}", cmd),
        }
    }
}

pub async fn help(args: Option<Vec<String>>) {
    let infos = get_help_info().await;
    if let Some(args) = args {
        if let Some(info) = infos.get(&args[0]) {
            async_print(format!("{}", info.value())).await;
            return;
        } else {
            async_print(format!("help info of {} not found", args[0])).await;
            return;
        }
    }

    let mut info_vec: Vec<String> = infos.iter().map(|info| info.value().to_owned()).collect::<Vec<String>>();
    info_vec.sort();
    for value in info_vec {
        async_print(format!("{}", value)).await;
    }
}

static HELP_INFO: OnceCell<DashMap<String, String>> = OnceCell::const_new();

async fn get_help_info() -> &'static DashMap<String, String> {
    HELP_INFO.get_or_init(|| async {
        let map = DashMap::new();
        map.insert("help".to_string(), "help      [args]                 : print help info".to_string());
        map.insert("delete".to_string(), "delete    [file_id]              : delete file from server".to_string());
        map.insert("download".to_string(), "download  [file_id] [file_path]  : download file from server".to_string());
        map.insert("exit".to_string(), "exit                             : exit terminal".to_string());
        map.insert("list_file".to_string(), "list_file [filter]               : list file in server, using filter as searching keyword".to_string());
        map.insert("login".to_string(), "login     [user_name] [password] : login to server".to_string());
        map.insert("register".to_string(), "register  [user_name] [password] : register to server".to_string());
        map.insert("upload".to_string(), "upload    [file_name] [path]     : upload file to server".to_string());
        map
    }).await
}

async fn input(user: Option<String>) -> (String, Option<Vec<String>>) {
    
    if let Some(user) = user {
        async_print(format!("{user} > ")).await;
    } else {
        async_print("user > ".to_string()).await;
    }

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut input = String::new();
    reader.read_line(&mut input).await.unwrap();
    let input = input.trim();
    let mut args = input.split_whitespace();
    let cmd = args.next().unwrap_or("").to_string();
    let args = args.map(|arg| arg.to_string()).collect::<Vec<_>>();
    let args = if args.len() > 0 {
        Some(args)
    } else {
        None
    };
    clear_terminal().await;
    (cmd, args)
}

async fn clear_terminal() {
    print!("\x1B[2J\x1B[1;1H");
}

pub async fn async_print(buffer: String) {
    let mut stdout = tokio::io::stdout();
    let buffer = format!("\n{buffer}");
    stdout.write_all(buffer.as_bytes()).await.unwrap();
    stdout.flush().await.unwrap();
}
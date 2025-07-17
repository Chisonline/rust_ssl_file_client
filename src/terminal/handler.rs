use tabled::{Table, Tabled};

use crate::{control::ControlBlock, file, terminal::{async_print, help}, user};

pub async fn login(block: &mut ControlBlock, args: Option<Vec<String>>) -> Option<String> {
    let (user_name, passwd) = match args {
        Some(args) => {
            if args.len() < 2 {
                help(Some(vec!["login".to_string()])).await;
                return None;
            }
            (args[0].to_owned(), args[1].to_owned())
        },
        None => {
            help(Some(vec!["login".to_string()])).await;
            return None;
        },
    };

    let resp = user::login::login(block, user_name.clone(), passwd).await;
    match resp {
        Ok(_) => {
            Some(user_name)
        },
        Err(e) => {
            async_print(format!("login failed: {:?}", e)).await;
            None
        }
    }
}

pub async fn register(block: &mut ControlBlock, args: Option<Vec<String>>) -> Option<String> {
    let (user_name, passwd) = match args {
        Some(args) => {
            if args.len() < 2 {
                help(Some(vec!["register".to_string()])).await;
                return None;
            }
            (args[0].to_owned(), args[1].to_owned())
        },
        None => {
            help(Some(vec!["register".to_string()])).await;
            return None;
        },
    };

    let resp = user::login::register(block, user_name.clone(), passwd).await;
    match resp {
        Ok(_) => {
            Some(user_name)
        },
        Err(e) => {
            async_print(format!("register failed: {:?}", e)).await;
            None
        }
    }
}

pub async fn delete(block: ControlBlock, args: Option<Vec<String>>) {
    let file_id = match args {
        Some(args) => {
            if args.len() < 1 {
                help(Some(vec!["delete".to_string()])).await;
                return;
            }
            args[0].to_owned()
        },
        None => {
            help(Some(vec!["delete".to_string()])).await;
            return;
        }
    };
    
    let file_id: i32 = match file_id.parse() {
        Ok(file_id) => file_id,
        Err(e) => {
            async_print(format!("illegal file_id: {:?}", e)).await;
            return;
        }
    };

    let resp = file::info::delete_file(block.clone(), file_id).await;
    match resp {
        Ok(_) => {
            async_print(format!("success")).await;
        },
        Err(e) => {
            async_print(format!("delete file failed: {:?}", e)).await;
        }
    }
}

pub async fn download(block: ControlBlock, args: Option<Vec<String>>) {
    let (file_id, target_path) = match args {
        Some(args) => {
            if args.len() < 2 {
                help(Some(vec!["download".to_string()])).await;
                return;
            }
            (args[0].to_owned(), args[1].to_owned())
        },
        None => {
            help(Some(vec!["download".to_string()])).await;
            return;
        }
    };

    let file_id: i32 = match file_id.parse() {
        Ok(file_id) => file_id,
        Err(e) => {
            async_print(format!("illegal file_id: {:?}", e)).await;
            return;
        }
    };

    let resp = file::download::download(block.clone(), file_id, &target_path).await;
    match resp {
        Ok(_) => {
            async_print(format!("download file success")).await;
        },
        Err(e) => {
            async_print(format!("download file failed: {:?}", e)).await;
        }
    }
}

pub async fn upload(block: ControlBlock, args: Option<Vec<String>>) {
    let (file_name, path) = match args {
        Some(args) => {
            if args.len() < 2 {
                help(Some(vec!["upload".to_string()])).await;
                return;
            }
            (args[0].to_owned(), args[1].to_owned())
        },
        None => {
            help(Some(vec!["upload".to_string()])).await;
            return;
        }
    };

    let resp = file::upload::upload(block, &file_name, path).await;
    match resp {
        Ok(_) => {
            async_print(format!("upload file success")).await;
        },
        Err(e) => {
            async_print(format!("upload file failed: {:?}", e)).await;
        }
    }
}

pub async fn list_file() {
    let resp = file::info::list_file("".to_string()).await;
    match resp {
        Ok(resp) => {
            #[derive(Tabled)]
            struct FileInfoDisplay {
                file_id: i32,
                file_name: String,
                file_size: i64,
                upload_time: String,
            }

            let file_info = resp.file_info.iter().map(|file_info| {
                FileInfoDisplay {
                    file_id: file_info.id,
                    file_name: file_info.file_name.clone(),
                    file_size: file_info.file_size,
                    upload_time: file_info.created_at.format("%Y0%m-%d %H:%M:%S").to_string(),
                }
            }).collect::<Vec<_>>();

            let table = Table::new(file_info);

            async_print(format!("{}", table)).await;
        },
        Err(e) => {
            async_print(format!("list file failed: {:?}", e)).await;
        }
    }
}


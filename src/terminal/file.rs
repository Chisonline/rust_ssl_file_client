use crate::terminal::help;

pub async fn login(args: Option<Vec<String>>) {
    let (user_name, passwd) = match args {
        Some(args) => {
            if args.len() < 2 {
                help(Some(vec!["login".to_string()])).await;
                return;
            }
            (args[0].to_owned(), args[1].to_owned())
        },
        None => return,
    };


}
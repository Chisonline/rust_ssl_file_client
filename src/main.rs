mod core;
mod utils;
mod control;
mod file;
mod user;
mod terminal;

#[tokio::main]
async fn main() -> ! {
    terminal::terminal().await
}
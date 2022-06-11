mod auth;
mod group_chat;
mod private_chat;

use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use dotenv;
use teloxide::{
    dispatching2::dialogue::{serializer::Json, SqliteStorage},
    prelude2::*,
};

use tokio::sync::Mutex;

async fn run() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    dotenv::var("USE_AUTH")
        .expect("Variable USE_AUTH should be set")
        .parse::<bool>()
        .expect("Should be 'true' or 'false'");
    let static_dir =
        PathBuf::from(dotenv::var("STATIC_DIR").expect("Variable STATIC_DIR should be set"));
    if !static_dir.is_dir() {
        panic!("Variable STATIC_DIT should contain valid path");
    }
    let conn = Arc::new(Mutex::new(db::establish_connection().await?));
    let bot = Bot::from_env().auto_send();
    let storage = SqliteStorage::open("db.sqlite", Json).await.unwrap();

    let handler = dptree::entry()
        .branch(
            dptree::filter(group_chat::filter_group_chats)
                .branch(group_chat::make_group_chat_branch()),
        )
        .branch(
            dptree::filter(private_chat::filter_private_chats)
                .branch(private_chat::make_private_chat_branch()),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![conn, storage, Arc::new(static_dir)])
        .default_handler(|upd| async move {
            dbg!(upd);
        })
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;

    Ok(())
}

async fn start_bot() -> Result<(), Box<dyn Error>> {
    teloxide::enable_logging!();
    log::info!("Running db migrations...");
    db::run_migrations().await?;
    log::info!("Starting bot...");
    run().await?;
    log::info!("Closing bot... Goodbye!");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    start_bot().await
}

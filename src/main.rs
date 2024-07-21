use std::path::PathBuf;
use log::{debug, log};
use mime::Mime;
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::{FileMeta, MediaKind, MessageCommon, MessageKind};
use tokio::fs;
use chrono::prelude::*;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let pdf_dir_path = std::path::PathBuf::from("uploads");

    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        let result = match msg.kind {
            MessageKind::Common(MessageCommon { from, media_kind: MediaKind::Document(doc), .. }) if doc.document.mime_type == Some(mime::APPLICATION_PDF) => {
                log::info!("Received a PDF document: {:?}", doc);

                let user = match from {
                    Some(user) =>  user.username.unwrap_or_else(|| user.id.to_string()),
                    None => String::from("very_weird_no_from_field_lmao"),
                };

                let file_name = format!("{}_{}_{}.pdf",
                    msg.date.format("%Y-%m-%d_%H-%M-%S").to_string(),
                    user,
                    doc.document.file_name.as_deref().unwrap_or("no_name"));

                let mut file_path = pdf_dir_path.clone();
                file_path.push(&file_name);

                let mut file_stream = fs::File::create(&file_path.to_str().unwrap()).await?;
                let file = bot.get_file(&doc.document.file.id).await?;
                bot.download_file(&file.path, &mut file_stream).await?;

                // print(&file_path).await.into()?;

                Ok(())
            }
            _ => {
                Err("Please send a PDF document")
            }
        };

        match result {
            Ok(()) => bot.send_message(msg.chat.id, "Please send a PDF document").await?,
            Err(e) => bot.send_message(msg.chat.id, e).await?
        };

        bot.send_dice(msg.chat.id).await?;
        Ok(())
    }).await;
}

async fn print(file_path: &PathBuf) -> Result<(), &str> {
    debug!("printing!!!");
    Ok(())
}
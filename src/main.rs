use log::*;
use std::ffi::{OsString};
use std::path::{Path, PathBuf};
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::{MediaKind, MessageCommon, MessageKind};
use tokio::fs;

#[derive(Clone, Debug)]
struct State {
    pub pdf_path: PathBuf,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let state = State {
        pdf_path: PathBuf::from("uploads"),
    };

    let bot = Bot::from_env();

    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let state = state.clone();

        async move {
            let result = match msg.kind {
                MessageKind::Common(MessageCommon { from, media_kind: MediaKind::Document(doc), .. }) if doc.document.mime_type == Some(mime::APPLICATION_PDF) => {
                    info!("Received a PDF document: {:?}", doc);

                    let user = match from {
                        Some(user) =>  user.username.unwrap_or_else(|| user.id.to_string()),
                        None => String::from("very_weird_no_from_field_lmao"),
                    };

                    let file_name = format!("{}_{}_{}.pdf",
                        msg.date.format("%Y-%m-%d_%H-%M-%S").to_string(),
                        user,
                        doc.document.file_name.as_deref().unwrap_or("no_name"));

                    let file_path = state.pdf_path.join(&file_name);

                    let mut file_stream = fs::File::create(&file_path).await.unwrap();
                    let file = bot.get_file(&doc.document.file.id).await?;
                    bot.download_file(&file.path, &mut file_stream).await?;

                    print(&file_path).await
                }
                _ => {
                    Err("Please send a PDF document".to_string())
                }
            };

            match result {
                Ok(()) => bot.send_message(msg.chat.id, "Please send a PDF document").await?,
                Err(e) => bot.send_message(msg.chat.id, e).await?
            };

            bot.send_dice(msg.chat.id).await?;
            Ok(())
        }
    }).await;
}

async fn print(file_path: &PathBuf) -> Result<(), String> {
    debug!("printing!!!");
    Ok(())
}

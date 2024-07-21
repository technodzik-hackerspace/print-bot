use log::*;
use std::path::PathBuf;
use std::process::Output;
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::{MediaKind, MessageCommon, MessageKind};
use tokio::fs;

#[derive(Clone, Debug)]
struct State {
    pub pdf_path: PathBuf,
    pub admin_group: String,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");

    let bot = Bot::from_env();

    let state = State {
        pdf_path: PathBuf::from("uploads"),
        admin_group: env!("ADMIN_GROUP_ID").to_string(),
    };

    fs::create_dir_all(&state.pdf_path).await.unwrap();

    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let state2 = state.clone();

        async move {
            //dbg!(&msg);
            let result = match msg.kind {
                MessageKind::Common(MessageCommon {
                                        media_kind: MediaKind::Document(doc),
                                        from: from_user,
                                        ..
                                    }) if doc.document.mime_type == Some(mime::APPLICATION_PDF) => {
                    info!("Received a PDF document: {:?}", doc);

                    let user = match from_user {
                        Some(user) =>  user.username.unwrap_or(user.id.to_string()),
                        None => String::from("very_weird_no_from_field_lmao"),
                    };

                    let file_name = format!("{}_{}_{}",
                        msg.date.format("%Y-%m-%d_%H-%M-%S").to_string(),
                        user,
                        doc.document.file_name.as_deref().unwrap_or_else(|| "no_name"));

                    let mut file_path = state2.pdf_path.join(&file_name);
                    file_path.set_extension("pdf");

                    let mut file_stream = fs::File::create(&file_path).await.unwrap();
                    let file = bot.get_file(&doc.document.file.id).await?;
                    bot.download_file(&file.path, &mut file_stream).await?;

                    bot.send_message(state2.admin_group.clone(),
                                     format!("New file {} sent to print by {}", &file_name, &user)
                    ).await?;
                    print(&file_path).await
                }
                _ => {
                    Err("Please send a PDF document".to_string())
                }
            };

            match result {
                Ok(out) => {
                    debug!("{}", String::from_utf8(out.stdout).unwrap());
                    bot.send_message(msg.chat.id, "File sent to the printer 🫡").await?
                },
                Err(e) => {
                    bot.send_message(state2.admin_group.clone(), &e).await?;
                    bot.send_message(msg.chat.id, "Something went wrong 😐").await?
                }
            };

            Ok(())
        }
    }).await;
}

async fn print(file_path: &PathBuf) -> Result<(), String> {
    debug!("printing!!!");
    Ok(())
}

use log::*;
use std::path::PathBuf;
use std::process::Output;
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::{DownloadError, RequestError};
use teloxide::types::{MediaKind, MessageCommon, MessageKind, User};
use tokio::fs;

#[derive(Clone, Debug)]
struct State {
    pub pdf_path: PathBuf,
    pub admin_group: String,
}

#[derive(Debug)]
enum PBError {
    LpError(String),
    PdfInfoError(String),
    WrongFileError(String),
    UnknownMessageKindError,
    NoDocumentError,
    RequestError(RequestError),
    DownloadError(DownloadError),
}

impl PBError {
    fn to_user_string(&self) -> String {
        match self {
            PBError::LpError(_e) => "Printer is not responding. Sorry".to_string(),
            PBError::PdfInfoError(e) => format!("Error getting the number of pages: {}", e),
            PBError::WrongFileError(mime) => format!("Please send a PDF document, not a {} 😐", mime),
            PBError::UnknownMessageKindError => "Something wrong with a printer. Sorry".to_string(),
            PBError::NoDocumentError => "Please send a PDF document 😐".to_string(),
            PBError::RequestError(_e) => "Failed to process your file. Try again or with another file".to_string(),
            PBError::DownloadError(_e) => "Failed to download your file. Try again or with another file".to_string(),
        }
    }

    fn to_admin_string(&self) -> String {
        match self {
            PBError::LpError(e) => format!("Error printing the document: {}", e),
            PBError::PdfInfoError(e) => format!("Error getting the number of pages: {}", e),
            PBError::WrongFileError(mime) => format!("Please send a PDF document, not a {} 😐", mime),
            PBError::UnknownMessageKindError => "Unknown message kind".to_string(),
            PBError::NoDocumentError => "Please send a PDF document 😐".to_string(),
            PBError::RequestError(e) => format!("Request error: {}", e),
            PBError::DownloadError(e) => format!("Download error: {}", e),
        }
    }
}

type PBResult<T> = Result<T, PBError>;

struct PBSuccess {
    pub pages: u32,
    pub file_name: String,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    // log::info!("Starting throw dice bot...");

    let bot = Bot::from_env();

    let state = State {
        pdf_path: PathBuf::from("uploads"),
        admin_group: std::env::var("ADMIN_GROUP_ID").expect("ADMIN_GROUP_ID env var is required"),
    };

    fs::create_dir_all(&state.pdf_path).await.unwrap();

    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let state2 = state.clone();

        async move {
            //dbg!(&msg);

            let user;

            let result = 'block: {
                match msg.kind {
                    MessageKind::Common(MessageCommon {
                                            media_kind: MediaKind::Document(doc),
                                            from: from_user,
                                            ..
                                        }) if doc.document.mime_type == Some(mime::APPLICATION_PDF) => {
                        user = get_user(from_user);
                        info!("Received a PDF document: {:?}", doc);

                        let original_name = doc.document.file_name.as_deref().unwrap_or("no_name");
                        let file_name = format!("{}_{}_{}",
                                                msg.date.format("%Y-%m-%d_%H-%M-%S").to_string(),
                                                user,
                                                original_name);

                        let mut file_path = state2.pdf_path.join(&file_name);
                        file_path.set_extension("pdf");

                        let mut file_stream = fs::File::create(&file_path).await.unwrap();

                        let file = bot.get_file(&doc.document.file.id).await
                            .map_err(|e| PBError::RequestError(e));

                        if let Err(e) = file {
                            break 'block Err(e);
                        }

                        let file = file.unwrap();

                        if let Err(e) = bot.download_file(&file.path, &mut file_stream).await
                            .map_err(|e| PBError::DownloadError(e)) {
                            break 'block Err(e);
                        }

                        let pages = match get_pages_number(&file_path).await {
                            Ok(pages) => pages,
                            Err(e) => break 'block Err(e)
                        };

                        if let Err(e) = print(&file_path).await {
                            break 'block Err(e);
                        }

                        Ok(PBSuccess { pages, file_name: original_name.to_string() })
                    },
                    MessageKind::Common(MessageCommon {
                                            media_kind: MediaKind::Document(doc),
                                            from: from_user,
                                            ..
                                        }) => {
                        user = get_user(from_user);
                        Err(
                            PBError::WrongFileError(
                                doc.document.mime_type
                                    .map(|mime| mime.to_string())
                                    .unwrap_or("unknown".to_string())
                            )
                        )
                    },
                    MessageKind::Common(MessageCommon {
                                            from: from_user,
                                            ..
                                        }) => {
                        user = get_user(from_user);

                        Err(PBError::NoDocumentError)
                    },

                    _ => {
                        user = "unknown user".to_string();
                        Err(PBError::UnknownMessageKindError)
                    }
                }
            };

            match result {
                Ok(pb_result) => {
                    debug!("Pages printed: {}", pb_result.pages);
                    let pages = match pb_result.pages {
                        1 => "1 page".to_string(),
                        n => format!("{} pages", n)
                    };
                    let success_message = format!("{} of file {} sent to the printer 🫡", pages, pb_result.file_name);

                    bot.send_message(msg.chat.id, &success_message).await?;
                    bot.send_message(
                        state2.admin_group.clone(),
                        format!("{}: {}", user, success_message)
                    ).await?
                },
                Err(pb_error) => {
                    bot.send_message(state2.admin_group.clone(), format!("{}: {}", user, pb_error.to_admin_string())).await?;
                    bot.send_message(msg.chat.id, pb_error.to_user_string()).await?
                },
            };

            Ok(())
        }
    }).await;
}

fn get_user(from_user: Option<User>) -> String {
    from_user
        .map(|user| user
            .username
            .unwrap_or(user.id.to_string()))
        .unwrap_or("unknown user".to_string())
}

async fn print(file_path: &PathBuf) -> PBResult<Output> {


    std::process::Command::new("lp")
        .arg(file_path)
        .output()
        .map_err(|e| PBError::LpError(e.to_string()))
}

async fn get_pages_number(file_path: &PathBuf) -> PBResult<u32> {
    std::process::Command::new("pdfinfo")
        .arg(file_path)
        .output()
        .map_err(|e| PBError::PdfInfoError(e.to_string()))
        .map(|output| {
            let output = String::from_utf8(output.stdout).unwrap();
            let pages = output.lines()
                .find(|line| line.starts_with("Pages:"))
                .map(|line| line.split_whitespace().last().unwrap())
                .map(|pages| pages.parse::<u32>().unwrap())
                .unwrap();
            pages
        })
}

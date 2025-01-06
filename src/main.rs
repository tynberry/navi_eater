use std::{io::Read, path::PathBuf};

use clap::Parser;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors possibly returned by this application.
#[derive(Debug, Error)]
enum AppError {
    #[error("could not find the token file at {path}")]
    TokenNotFound {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not read the token from token file")]
    TokenInvalid {
        #[source]
        source: std::io::Error,
    },
    #[error("could not find the message file at {path}")]
    MessageNotFound {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not parse the message")]
    MessageInvalid {
        #[source]
        source: serde_json::Error,
    },
    #[error("could not send the message")]
    MessageSendError,
}

/// Arguments of the application to load.
#[derive(Clone, Debug, Parser)]
struct CliArgs {
    /// Path to the file with a discord webhook.
    hook_file: PathBuf,
    /// File with the message to send.
    message: PathBuf,
}

/// Message to be sended.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Message {
    header: String,
    content: String,
    color: u32,
}

fn main() -> Result<(), AppError> {
    //load arguments
    let args = CliArgs::parse();
    //load token
    let mut hook_file =
        std::fs::File::open(args.hook_file.clone()).map_err(|err| AppError::TokenNotFound {
            path: args.hook_file,
            source: err,
        })?;
    let mut hook = String::new();
    hook_file
        .read_to_string(&mut hook)
        .map_err(|err| AppError::TokenInvalid { source: err })?;
    //load message
    let message_file =
        std::fs::File::open(args.message.clone()).map_err(|err| AppError::MessageNotFound {
            path: args.message,
            source: err,
        })?;
    let message: Message = serde_json::de::from_reader(message_file)
        .map_err(|err| AppError::MessageInvalid { source: err })?;

    //prepare body
    let body = format!(
        r#"{{"embeds":[{{"title":"{}","description":"{}","color":{}}}]}}"#,
        message.header, message.content, message.color
    );

    //make the request
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(hook.trim())
        .header("content-type", "application/json")
        .body(body)
        .send()
        .map_err(|_| AppError::MessageSendError)?;
    println!("{:?}", response);
    //all alright
    Ok(())
}

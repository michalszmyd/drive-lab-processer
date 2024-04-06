use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use crate::operations::{file_to_text, FileToTextError};

pub async fn resolve_routing(routing_key: &str, payload: &Vec<u8>) -> Result<String, JobError> {
    let captured_data = String::from_utf8_lossy(payload);

    match routing_key {
        "file_to_text" => file_to_text_job(&captured_data).await,
        _ => {
            info!("Incorrect routing key {}", routing_key);
            Err(JobError::NotFoundJobError(routing_key.to_string()))
        }
    }
}

#[derive(Serialize, Deserialize)]
struct FileToText<'a> {
    file_url: &'a str,
    extras: Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct FileToTextResult<'a> {
    extras: Value,
    text: &'a str,
}

#[derive(Debug, derive_more::From)]
pub enum JobError {
    JSONError(serde_json::Error),
    FileToTextError(FileToTextError),
    NotFoundJobError(String),
}

async fn file_to_text_job(raw_payload: &str) -> Result<String, JobError> {
    let parsed_data: FileToText = serde_json::from_str(raw_payload)?;

    let output = file_to_text(&parsed_data.file_url).await?;

    let result_data = FileToTextResult {
        text: &output,
        extras: parsed_data.extras,
    };

    let payload = serde_json::to_string(&result_data).unwrap();

    Ok(payload)
}

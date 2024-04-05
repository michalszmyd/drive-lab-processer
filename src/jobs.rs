use lapin::{options::BasicPublishOptions, BasicProperties, Channel};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use crate::{
    config::AppConfig,
    operations::{file_to_text, FileToTextError},
};

pub async fn resolve_routing(
    routing_key: &str,
    payload: &Vec<u8>,
    publish_channel: &Channel,
) -> Result<bool, JobError> {
    let captured_data = String::from_utf8_lossy(payload);

    info!("Performing with {:?}", &captured_data);

    match routing_key {
        "file_to_text" => file_to_text_job(&captured_data, publish_channel).await,
        _ => {
            info!("Incorrect routing key {}", routing_key);
            Ok(false)
        }
    }
}

#[derive(Serialize, Deserialize)]
struct FileToText {
    file_url: String,
    extras: Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct FileToTextResult {
    extras: Value,
    text: String,
}

#[derive(Debug, derive_more::From)]
pub enum JobError {
    JSONError(serde_json::Error),
    FileToTextError(FileToTextError),
}

async fn file_to_text_job(raw_payload: &str, publish_channel: &Channel) -> Result<bool, JobError> {
    let app_config = AppConfig::new();

    let parsed_data: FileToText = serde_json::from_str(raw_payload)?;

    let output = file_to_text(&parsed_data.file_url).await?;

    let result_data = FileToTextResult {
        text: output,
        extras: parsed_data.extras,
    };

    let payload = serde_json::to_string(&result_data).unwrap();

    let confirm = publish_channel
        .basic_publish(
            &app_config.file_to_text_job.publisher_exchange,
            &app_config.file_to_text_job.publisher_routing_key,
            BasicPublishOptions::default(),
            payload.as_bytes(),
            BasicProperties::default(),
        )
        .await
        .unwrap()
        .await
        .unwrap();

    info!("Confirmed: {:?}", confirm);

    Ok(true)
}

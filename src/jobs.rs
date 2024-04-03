use lapin::{options::BasicPublishOptions, BasicProperties, Channel};
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use tracing::info;

use crate::{config::AppConfig, operations::file_to_text};

pub async fn resolve_routing(routing_key: &str, payload: &Vec<u8>, publish_channel: &Channel) {
    let captured_data = String::from_utf8_lossy(payload);

    match routing_key {
        "file_to_text" => file_to_text_job(&captured_data, publish_channel).await,
        _ => {
            info!("Incorrect routing key {}", routing_key);
            false
        }
    };
}

#[derive(Serialize, Deserialize)]
struct FileToText {
    artifact_path: String,
    extras: Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct FileToTextResult {
    extras: Value,
    text: String,
}

async fn file_to_text_job(raw_payload: &str, publish_channel: &Channel) -> bool {
    let app_config = AppConfig::new();

    let parsed_data_result: Result<FileToText> = serde_json::from_str(raw_payload);

    match parsed_data_result {
        Ok(data) => {
            let output = file_to_text(&data.artifact_path);
            let result_data = FileToTextResult {
                text: output,
                extras: data.extras,
            };

            let payload = serde_json::to_string(&result_data).unwrap();

            dbg!(&payload);

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

            dbg!(confirm);

            true
        }
        Err(_) => {
            dbg!("Incorrect params");
            false
        }
    }
}

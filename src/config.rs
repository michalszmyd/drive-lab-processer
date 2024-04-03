use serde::Deserialize;
use std::fs;
use toml;
use tracing::info;

const CONFIG_FILE_PATH: &str = "./config.toml";

#[derive(Deserialize)]
struct ConfigToml {
    rabbit_mq: Option<ConfigTomlRabbitMQ>,
}

#[derive(Deserialize)]
struct ConfigTomlRabbitMQ {
    url: Option<String>,
    listen_queue: Option<String>,
    consumer_tag: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub rabbit_mq: RabbitMQConfig,
}

#[derive(Debug, Clone)]
pub struct RabbitMQConfig {
    pub url: String,
    pub listen_queue: String,
    pub consumer_tag: String,
}

impl AppConfig {
    fn empty_file_content() -> ConfigToml {
        info!("Empty config File");

        ConfigToml {
            rabbit_mq: Some(ConfigTomlRabbitMQ {
                url: None,
                listen_queue: None,
                consumer_tag: None,
            }),
        }
    }

    pub fn new() -> AppConfig {
        let toml_file_content = fs::read_to_string(CONFIG_FILE_PATH);

        let config_toml: ConfigToml = match toml_file_content {
            Ok(file_content) => {
                toml::from_str(&file_content).unwrap_or_else(|_| AppConfig::empty_file_content())
            }
            Err(_) => {
                info!("Error while opening file");

                AppConfig::empty_file_content()
            }
        };

        AppConfig {
            rabbit_mq: match config_toml.rabbit_mq {
                Some(config) => RabbitMQConfig {
                    url: config.url.unwrap_or("".to_string()),
                    listen_queue: config.listen_queue.unwrap_or("".to_string()),
                    consumer_tag: config.consumer_tag.unwrap_or("".to_string()),
                },
                None => RabbitMQConfig {
                    url: "".to_owned(),
                    listen_queue: "".to_owned(),
                    consumer_tag: "".to_owned(),
                },
            },
        }
    }
}

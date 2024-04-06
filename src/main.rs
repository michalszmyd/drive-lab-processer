use config::AppConfig;
use futures_lite::stream::StreamExt;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use tracing::info;

use crate::jobs::resolve_routing;

mod config;
mod jobs;
mod operations;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::init();

    let app_config = AppConfig::new();

    info!("{:?}", &app_config);

    let conn = Connection::connect(&app_config.rabbit_mq.url, ConnectionProperties::default())
        .await
        .unwrap();

    let channel = conn.create_channel().await.unwrap();

    let mut consumer = channel
        .basic_consume(
            &app_config.rabbit_mq.listen_queue,
            &app_config.rabbit_mq.consumer_tag,
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    info!(
        "RabbitMQ Listening to {}",
        app_config.rabbit_mq.listen_queue
    );

    while let Some(delivery) = consumer.next().await {
        let publish_channel_spawn = conn.create_channel().await.unwrap();
        let _ = tokio::spawn(async move {
            let delivery = delivery.expect("error in consumer");

            info!(
                "Received Routing key: {}, exchange: {}",
                &delivery.routing_key, &delivery.exchange,
            );

            let job = resolve_routing(
                &delivery.routing_key.as_str(),
                &delivery.data,
                &publish_channel_spawn,
            )
            .await;

            let _ = &publish_channel_spawn.close(200, "ok").await;

            match job {
                Ok(resolved) => {
                    if resolved {
                        let _ = &delivery.ack(BasicAckOptions::default()).await.expect("ack");

                        return;
                    } else {
                        info!("Error while performing Job {}", &delivery.routing_key)
                    }
                }
                Err(msg) => {
                    info!("Error: {:?}", &msg);

                    return;
                }
            };
        })
        .await;
    }
}

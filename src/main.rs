use config::AppConfig;
use futures_lite::stream::StreamExt;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties, Result};
use tokio::time;
use tracing::info;

use crate::jobs::resolve_routing;

mod config;
mod jobs;
mod operations;

#[tokio::main]
async fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::init();

    let app_config = AppConfig::new();

    dbg!(&app_config);

    let addr = app_config.rabbit_mq.url;
    let conn = Connection::connect(&addr, ConnectionProperties::default())
        .await
        .unwrap();

    let _ = consumer(&conn).await;
}

async fn consumer(conn: &Connection) -> Result<()> {
    let app_config = AppConfig::new();
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

    let publish_channel = conn.create_channel().await.unwrap();

    loop {
        info!(
            "RabbitMQ Listneing to {}",
            app_config.rabbit_mq.listen_queue
        );

        while let Some(delivery) = consumer.next().await {
            let publish_channel_spawn = publish_channel.clone();

            tokio::spawn(async move {
                info!("Received message, parsing");

                let delivery = delivery.expect("error in consumer");

                info!(
                    "Received Routing key: {}, exchange: {},",
                    delivery.routing_key, delivery.exchange,
                );

                resolve_routing(
                    &delivery.routing_key.as_str(),
                    &delivery.data,
                    &publish_channel_spawn,
                )
                .await;

                delivery.ack(BasicAckOptions::default()).await.expect("ack");
            });
        }

        time::sleep(time::Duration::from_secs(1)).await;
    }
}

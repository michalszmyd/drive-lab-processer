use std::sync::Arc;

use config::AppConfig;
use futures_lite::stream::StreamExt;
use lapin::{
    message::Delivery, options::*, types::FieldTable, BasicProperties, Connection,
    ConnectionProperties,
};
use tracing::info;

use crate::jobs::resolve_routing;

mod config;
mod jobs;
mod operations;

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    std::env::set_var("JEMALLOC_SYS_WITH_MALLOC_CONF", "background_thread:true,narenas:1,tcache:false,dirty_decay_ms:0,muzzy_decay_ms:0,abort_conf:true");

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    let app_config = AppConfig::new();

    let conn = Connection::connect(&app_config.rabbit_mq.url, ConnectionProperties::default())
        .await
        .unwrap();

    let channel = &conn.create_channel().await.unwrap();

    let mut consumer = channel
        .basic_consume(
            &app_config.rabbit_mq.listen_queue,
            &app_config.rabbit_mq.consumer_tag,
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    while let Some(delivery) = consumer.next().await {
        let arc_channel = Arc::new(channel.clone());
        let arc_app_config = Arc::new(app_config.clone());
        let delivery = delivery.expect("error in consumer");
        info!("{:?}", delivery.routing_key);

        tokio::spawn(async move {
            let job_info = consume_delivery(delivery).await;

            match job_info {
                Some(payload) => {
                    let publish = arc_channel.basic_publish(
                        &arc_app_config.file_to_text_job.publisher_exchange,
                        &arc_app_config.file_to_text_job.publisher_routing_key,
                        BasicPublishOptions::default(),
                        &payload.as_bytes(),
                        BasicProperties::default(),
                    );

                    let _ = publish.await;
                }
                None => {}
            }
        });
    }
}

async fn consume_delivery(delivery: Delivery) -> Option<String> {
    let job = resolve_routing(&delivery.routing_key.as_str(), &delivery.data).await;

    match job {
        Ok(resolved) => {
            let _ = &delivery.ack(BasicAckOptions::default()).await.expect("ack");

            Some(resolved)
        }
        Err(msg) => {
            info!("Error: {:?}", &msg);

            None
        }
    }
}

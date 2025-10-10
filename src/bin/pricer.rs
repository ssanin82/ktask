use tonic::{transport::Server, Response, Status};
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc;
use futures_core::Stream;
use std::pin::Pin;
use std::time::Duration;
use ktask::okx::run as run_okx;
use ktask::binance::run as run_bnc;
use ktask::order_book::OrderBook;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;

const PUB_IVAL_SEC: u64 = 1;

pub mod pubsub {
    tonic::include_proto!("pubsub");
}

use pubsub::publisher_server::{Publisher, PublisherServer};
use pubsub::{Message, SubscribeRequest, Level};

type ResponseStream = Pin<Box<dyn Stream<Item = Result<Message, Status>> + Send>>;

pub struct MyPublisher {
    order_book: Arc<Mutex<OrderBook>>,
}

impl MyPublisher {
    fn new(ob: Arc<Mutex<OrderBook>>) -> Self {
        MyPublisher {
            order_book: ob,
        }
    }
}

#[tonic::async_trait]
impl Publisher for MyPublisher {
    type SubscribeStream = ResponseStream;

    async fn subscribe(
        &self,
        request: tonic::Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let topic = request.into_inner().topic;
        println!("Subscriber joined for topic '{}'", topic);

        if topic != "prices" {
            println!("Rejected subscription for topic '{}'", topic);
            return Err(Status::permission_denied(format!(
                "Topic '{}' is not allowed. Only 'news' is available.",
                topic
            )));
        }

        let (tx, rx) = mpsc::channel(10);
        let ob_clone = Arc::clone(&self.order_book);
        tokio::spawn(async move {
            let ob = ob_clone.lock().await;
            loop {
                // sending the message -----------
                let msg = Message {
                    topic: "prices".to_string(),
                    // TODO
                    spread: "123".to_string(),
                    bids: vec![
                        Level { price: "456".to_string(), size: "789".to_string() },
                        Level { price: "654".to_string(), size: "987".to_string() },
                    ],
                    asks: vec![
                        Level { price: "456".to_string(), size: "789".to_string() },
                        Level { price: "654".to_string(), size: "987".to_string() },
                    ],
                };
                if tx.send(Ok(msg)).await.is_err() {
                    println!("Client disconnected from '{}'", topic);
                    break;
                }
                tokio::time::sleep(Duration::from_secs(PUB_IVAL_SEC)).await;
                //
                ob.print_detailed();
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx)) as Self::SubscribeStream))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let ob = Arc::new(Mutex::new(OrderBook::new()));

    let ob1 = Arc::clone(&ob);
    tokio::spawn(async move { run_bnc(ob1).await });
    let ob2 = Arc::clone(&ob);
    tokio::spawn(async move { run_okx(ob2).await });

    let addr = "127.0.0.1:50051".parse().unwrap();
    let ob_pub = Arc::clone(&ob);
    let publisher = MyPublisher::new(ob_pub);

    println!("Publisher gRPC server listening on {}", addr);
    Server::builder()
        .add_service(PublisherServer::new(publisher))
        .serve(addr)
        .await?;

    Ok(())
}

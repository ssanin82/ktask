use tonic::{transport::Server, Response, Status};
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use futures_core::Stream;
use std::pin::Pin;
use std::time::Duration;
use ktask::okx::run as run_okx;
use ktask::binance::run as run_bnc;
use ktask::order_book::{OrderBook, Side, PRICE_PRECISION, SIZE_PRECISION};
use ktask::helpers::itos;
use ktask::api_server::run_api_server;
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
            loop {
                {
                    let ob = ob_clone.lock().await;
                    let msg = Message {
                        topic: "prices".to_string(),
                        spread: itos(ob.get_spread().unwrap().2, PRICE_PRECISION),
                        bids: ob.top_n(Side::Bid, 10).iter()
                            .map(|x| Level {
                                price: itos(x.price, PRICE_PRECISION),
                                size: itos(x.total, SIZE_PRECISION),
                            }).collect(),
                        asks: ob.top_n(Side::Ask, 10).iter().rev()
                            .map(|x| Level {
                                price: itos(x.price, PRICE_PRECISION),
                                size: itos(x.total, SIZE_PRECISION),
                            }).collect(),
                    };
                    if tx.send(Ok(msg)).await.is_err() {
                        println!("Client disconnected from '{}'", topic);
                        break;
                    }
                    ob.print_detailed();
                }
                tokio::time::sleep(Duration::from_secs(PUB_IVAL_SEC)).await;
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx)) as Self::SubscribeStream))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let ob = Arc::new(Mutex::new(OrderBook::new()));
    
    // Create broadcast channel for WebSocket updates
    let (tx, _) = broadcast::channel::<String>(100);

    let ob1 = Arc::clone(&ob);
    tokio::spawn(async move { run_bnc(ob1).await });
    let ob2 = Arc::clone(&ob);
    tokio::spawn(async move { run_okx(ob2).await });

    // Start API server (WebSocket + REST)
    let ob_api = Arc::clone(&ob);
    let tx_api = tx.clone();
    tokio::spawn(async move {
        if let Err(e) = run_api_server(ob_api, tx_api).await {
            eprintln!("API server error: {}", e);
        }
    });

    // Start publishing task for WebSocket
    let ob_pub = Arc::clone(&ob);
    let tx_pub = tx.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await; // 10 updates per second
            let snapshot = {
                let ob = ob_pub.lock().await;
                ob.create_snapshot(50)
            };
            if let Ok(json) = serde_json::to_string(&snapshot) {
                let _ = tx_pub.send(json);
            }
        }
    });

    // Keep gRPC server for backward compatibility (moved to 50052 to free 50051 for API server)
    let addr = "127.0.0.1:50052".parse().unwrap();
    let publisher = MyPublisher::new(Arc::clone(&ob));
    println!("Publisher gRPC server listening on {}", addr);
    Server::builder()
        .add_service(PublisherServer::new(publisher))
        .serve(addr)
        .await?;

    Ok(())
}

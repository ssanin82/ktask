// use tonic::{transport::Server, Response, Status};
// use futures_core::Stream;
// use tokio_stream::wrappers::ReceiverStream;
// use tokio::sync::mpsc;
// use std::pin::Pin;
// use std::time::Duration;

// pub mod pubsub {
//     tonic::include_proto!("pubsub");
// }

// use pubsub::publisher_server::{Publisher, PublisherServer};
// use pubsub::{Message, SubscribeRequest};

// type ResponseStream = Pin<Box<dyn Stream<Item = Result<Message, Status>> + Send>>;

// #[derive(Default)]
// pub struct MyPublisher {}

// #[tonic::async_trait]
// impl Publisher for MyPublisher {
//     type SubscribeStream = ResponseStream;

//     async fn subscribe(
//         &self,
//         request: tonic::Request<SubscribeRequest>,
//     ) -> Result<Response<Self::SubscribeStream>, Status> {
//         let topic = request.into_inner().topic;
//         println!("Subscriber joined for topic '{}'", topic);

//         let (tx, rx) = mpsc::channel(10);

//         // Spawn a task that periodically sends messages
//         tokio::spawn(async move {
//             let mut counter = 0;
//             loop {
//                 let msg = Message {
//                     topic: topic.clone(),
//                     content: format!("Breaking news #{}", counter),
//                 };
//                 if tx.send(Ok(msg)).await.is_err() {
//                     println!("Client disconnected from '{}'", topic);
//                     break;
//                 }
//                 counter += 1;
//                 tokio::time::sleep(Duration::from_secs(2)).await;
//             }
//         });

//         Ok(Response::new(Box::pin(ReceiverStream::new(rx)) as Self::SubscribeStream))
//     }
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let addr = "127.0.0.1:50051".parse().unwrap();
//     let publisher = MyPublisher::default();

//     println!("Publisher gRPC server listening on {}", addr);

//     Server::builder()
//         .add_service(PublisherServer::new(publisher))
//         .serve(addr)
//         .await?;

//     Ok(())
// }

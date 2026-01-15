use tonic::Request;
use pubsub::publisher_client::PublisherClient;
use pubsub::SubscribeRequest;

pub mod pubsub {
    tonic::include_proto!("pubsub");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PublisherClient::connect("http://127.0.0.1:50052").await?;
    println!("Connected to gRPC publisher.");

    let request = Request::new(SubscribeRequest {
        topic: "prices".into(),
    });

    let mut stream = client.subscribe(request).await?.into_inner();

    while let Some(msg) = stream.message().await? {
        println!("Received -> SPREAD: {}", msg.spread);
        println!("\nASKS:");
        for val in &msg.asks {
            println!("{} -> {}", val.price, val.size);
        }
        println!("\nBIDS:");
        for val in &msg.bids {
            println!("{} -> {}", val.price, val.size);
        }
        println!("");
        println!("");
    }

    Ok(())
}

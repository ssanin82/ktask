use prost::Message;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

pub mod pubsub {
    include!(concat!(env!("OUT_DIR"), "/pubsub.rs"));
}

fn handle_client(mut stream: TcpStream) {
    let mut counter = 0;
    loop {
        let msg = pubsub::Message {
            topic: "news".to_string(),
            content: format!("Breaking news #{}", counter),
        };

        let mut buf = Vec::new();
        msg.encode(&mut buf).unwrap();

        // Prepend message length (4 bytes, big-endian)
        let len = (buf.len() as u32).to_be_bytes();
        if stream.write_all(&len).is_err() || stream.write_all(&buf).is_err() {
            println!("Client disconnected");
            break;
        }

        println!("Sent: {}", msg.content);
        counter += 1;
        thread::sleep(Duration::from_secs(2));
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5000")?;
    println!("Publisher running on 127.0.0.1:5000");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New subscriber connected");
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
    Ok(())
}

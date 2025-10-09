// use prost::Message;
// use std::io::{Read};
// use std::net::TcpStream;

// pub mod pubsub {
//     include!(concat!(env!("OUT_DIR"), "/pubsub.rs"));
// }

// fn main() -> std::io::Result<()> {
//     let mut stream = TcpStream::connect("127.0.0.1:5000")?;
//     println!("Connected to publisher.");

//     loop {
//         // Read length prefix
//         let mut len_buf = [0u8; 4];
//         if stream.read_exact(&mut len_buf).is_err() {
//             println!("Publisher closed connection");
//             break;
//         }
//         let msg_len = u32::from_be_bytes(len_buf) as usize;

//         // Read message bytes
//         let mut buf = vec![0u8; msg_len];
//         if stream.read_exact(&mut buf).is_err() {
//             println!("Stream ended");
//             break;
//         }

//         let msg = pubsub::Message::decode(&*buf).unwrap();
//         println!("Received [{}]: {}", msg.topic, msg.content);
//     }

//     Ok(())
// }

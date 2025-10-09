use tungstenite::{client::IntoClientRequest, WebSocket, Message};
use tungstenite::client_tls;
use url::Url;
use std::net::TcpStream;
use rustls::crypto::aws_lc_rs;
use rustls::crypto::CryptoProvider;

fn main() {
    let _ = CryptoProvider::install_default(aws_lc_rs::default_provider());

    let url = Url::parse("wss://stream.binance.com:9443/ws/ethbtc@depth20").unwrap();
    let req = url.as_str().into_client_request().unwrap();
    let domain = url.domain().unwrap();
    let port = url.port().unwrap();

    let tcp = TcpStream::connect((domain, port)).unwrap();
    let (mut socket, _) = client_tls(req, tcp).unwrap();
    println!("Connected to Binance depth20 feed");
    loop {
        match socket.read_message() {
            Ok(Message::Text(txt)) => println!("{}", txt),
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
}

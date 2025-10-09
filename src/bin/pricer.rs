use clap::Parser;
use std::thread;
use std::time::Duration;
// use ktask::okx::run as run_okx;
use ktask::binance::run as run_bnc;
use ktask::order_book::OrderBook;
use std::sync::{Arc, Mutex};

#[derive(Parser, Debug)]
#[command(name = "server")]
#[command(about = "Reads host and port arguments", long_about = None)]
struct Args {
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    #[arg(long, default_value_t = 8080)]
    port: i32,
}

fn main() {
    let args = Args::parse();
    println!("Host: {}", args.host);
    println!("Port: {}", args.port);

    let ob = Arc::new(Mutex::new(OrderBook::new()));

    let ob1 = Arc::clone(&ob);
    let feed_bnc = thread::spawn(move || {
        run_bnc(ob1);
    });
    // let ob2 = Arc::clone(&ob);
    // let feed_okx = thread::spawn(move || {
    //     run_okx(ob2);
    // });
    for _i in 1..=5 {
        thread::sleep(Duration::from_millis(3000));
        ob.lock().unwrap().print();
    }
    feed_bnc.join().unwrap();
    // feed_okx.join().unwrap();
    println!("Done!");
}

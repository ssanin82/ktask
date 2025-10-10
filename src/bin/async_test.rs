use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use chrono::Local;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub total: i32,
    pub by_source: HashMap<String, i32>,
}

impl PriceLevel {
    fn new() -> Self {
        Self { total: 0, by_source: HashMap::new() }
    }
}

#[derive(Debug)]
pub struct SyntheticOrderBook {
    bids: BTreeMap<i32, PriceLevel>,
    asks: BTreeMap<i32, PriceLevel>,
}

#[derive(Debug, Clone)]
pub struct PriceLevelSnapshot {
    pub price: i32,
    pub total: i32,
    pub by_source: HashMap<String, i32>,
}

impl SyntheticOrderBook {
    pub fn new() -> Self {
        Self { bids: BTreeMap::new(), asks: BTreeMap::new() }
    }

    pub fn apply_update(&mut self, source: &str, side: Side, price: i32, size: i32) {
        let book_map = match side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks,
        };

        if size == 0 {
            if let Some(level) = book_map.get_mut(&price) {
                if level.by_source.remove(source).is_some() {
                    level.total = level.by_source.values().copied().sum();
                }
                if level.by_source.is_empty() {
                    book_map.remove(&price);
                }
            }
        } else {
            let level = book_map.entry(price).or_insert_with(PriceLevel::new);
            level.by_source.insert(source.to_string(), size);
            level.total = level.by_source.values().copied().sum();
        }
    }

    pub fn get_spread(&self) -> Option<(i32, i32, i32)> {
        let best_bid = self.bids.iter().rev().next().map(|(k, _v)| *k);
        let best_ask = self.asks.iter().next().map(|(k, _v)| *k);
        match (best_bid, best_ask) {
            (Some(b), Some(a)) => Some((b, a, a - b)),
            _ => None,
        }
    }

    pub fn top_n(&self, side: Side, n: usize) -> Vec<PriceLevelSnapshot> {
        let mut out = Vec::new();
        match side {
            Side::Bid => {
                for (k, v) in self.bids.iter().rev().take(n) {
                    out.push(PriceLevelSnapshot { price: *k, total: v.total, by_source: v.by_source.clone() });
                }
            }
            Side::Ask => {
                for (k, v) in self.asks.iter().take(n) {
                    out.push(PriceLevelSnapshot { price: *k, total: v.total, by_source: v.by_source.clone() });
                }
            }
        }
        out
    }

    pub fn print_book(&self) {
        let now = Local::now();
        println!("\n==============================");
        println!("Synthetic Order Book Snapshot @ {}", now.format("%H:%M:%S"));
        println!("==============================");

        let asks = self.top_n(Side::Ask, 10);
        let bids = self.top_n(Side::Bid, 10);

        println!("   Price      AskSize");
        for level in asks.iter().rev() {
            println!("{:>8} | {:>8}", level.price, level.total);
        }

        println!("------------------------------");
        println!("   Price      BidSize");
        for level in &bids {
            println!("{:>8} | {:>8}", level.price, level.total);
        }

        if let Some((bid, ask, spread)) = self.get_spread() {
            println!("------------------------------");
            println!("Best Bid: {} | Best Ask: {} | Spread: {}", bid, ask, spread);
        } else {
            println!("Spread: Not available");
        }
        println!("==============================\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_remove_levels() {
        let mut book = SyntheticOrderBook::new();
        book.apply_update("A", Side::Bid, 100, 5);
        book.apply_update("B", Side::Bid, 100, 3);
        assert_eq!(book.bids.get(&100).unwrap().total, 8);

        book.apply_update("A", Side::Bid, 100, 0);
        assert_eq!(book.bids.get(&100).unwrap().total, 3);

        book.apply_update("B", Side::Bid, 100, 0);
        assert!(book.bids.get(&100).is_none());
    }

    #[test]
    fn test_spread_calculation() {
        let mut book = SyntheticOrderBook::new();
        book.apply_update("A", Side::Bid, 99, 5);
        book.apply_update("A", Side::Ask, 101, 4);
        let spread = book.get_spread().unwrap();
        assert_eq!(spread, (99, 101, 2));
    }

    #[test]
    fn test_top_n_levels() {
        let mut book = SyntheticOrderBook::new();
        for i in 90..=100 {
            book.apply_update("A", Side::Bid, i, 1);
            book.apply_update("A", Side::Ask, i + 20, 2);
        }
        let top_bids = book.top_n(Side::Bid, 5);
        let top_asks = book.top_n(Side::Ask, 5);
        assert_eq!(top_bids.len(), 5);
        assert_eq!(top_asks.len(), 5);
        assert!(top_bids[0].price > top_bids[4].price);
        assert!(top_asks[0].price < top_asks[4].price);
    }
}

#[tokio::main]
async fn main() {
    let book = Arc::new(Mutex::new(SyntheticOrderBook::new()));

    let book_clone_a = Arc::clone(&book);
    let task_a = tokio::spawn(async move {
        let mut price = 100;
        loop {
            {
                let mut b = book_clone_a.lock().unwrap();
                b.apply_update("MarketA", Side::Bid, price, 5);
                b.apply_update("MarketA", Side::Ask, price + 10, 3);
            }
            price += 1;
            sleep(Duration::from_millis(500)).await;
        }
    });

    let book_clone_b = Arc::clone(&book);
    let task_b = tokio::spawn(async move {
        let mut price = 105;
        loop {
            {
                let mut b = book_clone_b.lock().unwrap();
                b.apply_update("MarketB", Side::Bid, price, 4);
                b.apply_update("MarketB", Side::Ask, price + 8, 2);
            }
            price -= 1;
            sleep(Duration::from_millis(700)).await;
        }
    });

    let book_clone_c = Arc::clone(&book);
    let task_c = tokio::spawn(async move {
        loop {
            {
                let b = book_clone_c.lock().unwrap();
                b.print_book();
            }
            sleep(Duration::from_secs(2)).await;
        }
    });

    let _ = tokio::join!(task_a, task_b, task_c);
}

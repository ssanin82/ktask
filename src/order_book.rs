use std::collections::{BTreeMap, HashMap};
use chrono::Utc;

pub const PRICE_PRECISION: usize = 5;
pub const SIZE_PRECISION: usize = 6;

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
pub struct OrderBook {
    // bids: sorted descending by price -> use BTreeMap; iterate in reverse for bids
    bids: BTreeMap<i32, PriceLevel>,
    // asks: sorted ascending by price
    asks: BTreeMap<i32, PriceLevel>,
}

#[derive(Debug, Clone)]
pub struct PriceLevelSnapshot {
    pub price: i32,
    pub total: i32,
    pub by_source: HashMap<String, i32>,
}

impl OrderBook {
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

    /// return best bid price, best ask price and spread (ask - bid) if both exist
    pub fn get_spread(&self) -> Option<(i32, i32, i32)> {
        let best_bid = self.bids.iter().rev().next().map(|(k, _v)| *k);
        let best_ask = self.asks.iter().next().map(|(k, _v)| *k);
        match (best_bid, best_ask) {
            (Some(b), Some(a)) => Some((b, a, a - b)),
            _ => None,
        }
    }

    /// Return top n snapshots for the given side. For bids returns highest prices first; for asks lowest first.
    pub fn top_n(&self, side: Side, n: usize) -> Vec<PriceLevelSnapshot> {
        let mut out = Vec::new();
        match side {
            Side::Bid => {
                for (k, v) in self.bids.iter().rev().take(n) {
                    out.push(PriceLevelSnapshot {
                        price: *k,
                        total: v.total,
                        by_source: v.by_source.clone()
                    });
                }
            }
            Side::Ask => {
                for (k, v) in self.asks.iter().take(n) {
                    out.push(PriceLevelSnapshot {
                        price: *k,
                        total: v.total,
                        by_source: v.by_source.clone()
                    });
                }
            }
        }
        out
    }

    pub fn print(&self) {
        let now = Utc::now();
        println!("Current timestamp: {}", now);
        println!("ASKS:");
        for pls in self.top_n(Side::Ask, 10).iter().rev() {
            println!("{} -> {}", pls.price, pls.total);
        }
        println!("");
        println!("BIDS:");
        for pls in self.top_n(Side::Bid, 10).iter() {
            println!("{} -> {}", pls.price, pls.total);
        }
        println!("");
        println!("SPREAD: {}", self.get_spread().unwrap().2);
        println!("");
    }
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_flow() {
        let mut book = OrderBook::new();

        // Market A posts a bid at 100 size 5
        book.apply_update("MarketA", Side::Bid, 100, 5);
        // Market B posts a bid at same price 100 size 3
        book.apply_update("MarketB", Side::Bid, 100, 3);
        // Combined at 100 should be 8
        let top_bids = book.top_n(Side::Bid, 10);
        assert_eq!(top_bids.len(), 1);
        assert_eq!(top_bids[0].price, 100);
        assert_eq!(top_bids[0].total, 8);

        // Market A deletes its 100 bid (size 0) -> remaining should be MarketB's 3
        book.apply_update("MarketA", Side::Bid, 100, 0);
        let top_bids = book.top_n(Side::Bid, 10);
        assert_eq!(top_bids.len(), 1);
        assert_eq!(top_bids[0].total, 3);

        // Market B deletes too -> book empty
        book.apply_update("MarketB", Side::Bid, 100, 0);
        assert!(book.top_n(Side::Bid, 10).is_empty());

        // add asks
        book.apply_update("MarketA", Side::Ask, 101, 2);
        book.apply_update("MarketB", Side::Ask, 102, 1);
        // no bids yet, so spread should be None
        assert!(book.get_spread().is_none());

        // add a bid to get a spread
        book.apply_update("MarketA", Side::Bid, 100, 1);
        let s = book.get_spread().expect("now both exist");
        assert!(s.2 > 0);
    }
}

// fn main() {
//     let mut book = OrderBook::new();
//     book.apply_update("A", Side::Bid, 100, 5);
//     book.apply_update("B", Side::Bid, 100, 3);
//     book.apply_update("A", Side::Ask, 101, 2);
//     book.apply_update("A", Side::Ask, 102, 2);
//     book.apply_update("B", Side::Ask, 103, 3);
//     if let Some((b, a, spread)) = book.get_spread() {
//         println!("best bid {} best ask {} spread {}", b, a, spread);
//     }
//     let bids = book.top_n(Side::Bid, 10);
//     let asks = book.top_n(Side::Ask, 10);
//     println!("bids: {:?}\nasks: {:?}", bids, asks);
// }

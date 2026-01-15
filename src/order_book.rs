use std::collections::{BTreeMap, HashMap};
use chrono::{Utc, DateTime};
use serde::{Serialize, Deserialize};

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
    upd_count: HashMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevelSnapshot {
    pub price: i32,
    pub total: i32,
    pub by_source: HashMap<String, i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    pub timestamp: DateTime<Utc>,
    pub spread: Option<i32>,
    pub best_bid: Option<i32>,
    pub best_ask: Option<i32>,
    pub mid_price: Option<i32>,
    pub bids: Vec<PriceLevelSnapshot>,
    pub asks: Vec<PriceLevelSnapshot>,
    pub total_bid_volume: i32,
    pub total_ask_volume: i32,
    pub update_counts: HashMap<String, i32>,
    pub vwap_bid: Option<f64>,
    pub vwap_ask: Option<f64>,
    pub depth_bid_5bps: i32,
    pub depth_ask_5bps: i32,
    pub depth_bid_10bps: i32,
    pub depth_ask_10bps: i32,
    pub imbalance: Option<f64>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            upd_count: HashMap::new(),
        }
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
            let count = self.upd_count.entry(source.to_string()).or_insert(0);
            *count += 1;

            // println!(
            //     "UPDATE COUNT: Binance={}, OKX={}\n\n",
            //     self.upd_count.get(&String::from("BINANCE")).unwrap_or(&0),
            //     self.upd_count.get(&String::from("OKX")).unwrap_or(&0)
            // );
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

    pub fn print_detailed(&self) {
        let now = Utc::now();
        println!("Current timestamp: {}", now);

        println!("ASKS:");
        println!("{:<10} {:<10} {:<10} {:<10}", "Price", "Binance", "OKX", "Total");
        for pls in self.top_n(Side::Ask, 10).iter().rev() {
            let binance_size = pls.by_source.get("BINANCE").copied().unwrap_or(0);
            let okx_size = pls.by_source.get("OKX").copied().unwrap_or(0);
            println!(
                "{:<10} {:<10} {:<10} {:<10}",
                pls.price, binance_size, okx_size, pls.total
            );
        }

        println!("\nBIDS:");
        println!("{:<10} {:<10} {:<10} {:<10}", "Price", "Binance", "OKX", "Total");
        for pls in self.top_n(Side::Bid, 10).iter() {
            let binance_size = pls.by_source.get("BINANCE").copied().unwrap_or(0);
            let okx_size = pls.by_source.get("OKX").copied().unwrap_or(0);
            println!(
                "{:<10} {:<10} {:<10} {:<10}",
                pls.price, binance_size, okx_size, pls.total
            );
        }

        if let Some((_bid, _ask, spread)) = self.get_spread() {
            println!("\nSPREAD: {}", spread);
        } else {
            println!("\nSPREAD: N/A");
        }

        println!(
            "UPDATE COUNT: Binance={}, OKX={}\n\n",
            self.upd_count.get(&String::from("BINANCE")).unwrap_or(&0),
            self.upd_count.get(&String::from("OKX")).unwrap_or(&0)
        );
    }

    /// Calculate VWAP for bids or asks
    fn calculate_vwap(&self, side: Side, levels: usize) -> Option<f64> {
        let snapshots = self.top_n(side, levels);
        if snapshots.is_empty() {
            return None;
        }
        let total_value: i64 = snapshots.iter().map(|s| s.price as i64 * s.total as i64).sum();
        let total_volume: i64 = snapshots.iter().map(|s| s.total as i64).sum();
        if total_volume == 0 {
            return None;
        }
        Some(total_value as f64 / total_volume as f64)
    }

    /// Calculate cumulative volume within basis points from mid price
    fn calculate_depth(&self, side: Side, bps: i32) -> Option<i32> {
        let spread_info = self.get_spread()?;
        let (best_bid, best_ask, _spread) = spread_info;
        let mid_price = (best_bid + best_ask) / 2;
        
        let price_offset = (mid_price as f64 * bps as f64 / 10000.0) as i32;
        let target_price = match side {
            Side::Bid => best_bid - price_offset,
            Side::Ask => best_ask + price_offset,
        };

        let snapshots = self.top_n(side, 1000);
        let mut total = 0;
        for snapshot in snapshots {
            let include = match side {
                Side::Bid => snapshot.price >= target_price,
                Side::Ask => snapshot.price <= target_price,
            };
            if include {
                total += snapshot.total;
            } else {
                break;
            }
        }
        Some(total)
    }

    /// Get total volume for top N levels
    fn total_volume(&self, side: Side, n: usize) -> i32 {
        self.top_n(side, n).iter().map(|s| s.total).sum()
    }

    /// Create a comprehensive snapshot for API/WebSocket
    pub fn create_snapshot(&self, depth: usize) -> OrderBookSnapshot {
        let timestamp = Utc::now();
        let spread_info = self.get_spread();
        let (best_bid, best_ask, spread) = spread_info.unwrap_or((0, 0, 0));
        let mid_price = spread_info.map(|(b, a, _)| (b + a) / 2);

        let bids = self.top_n(Side::Bid, depth);
        let asks = self.top_n(Side::Ask, depth);
        
        let total_bid_volume = self.total_volume(Side::Bid, depth);
        let total_ask_volume = self.total_volume(Side::Ask, depth);
        
        let vwap_bid = self.calculate_vwap(Side::Bid, depth);
        let vwap_ask = self.calculate_vwap(Side::Ask, depth);
        
        let depth_bid_5bps = self.calculate_depth(Side::Bid, 5).unwrap_or(0);
        let depth_ask_5bps = self.calculate_depth(Side::Ask, 5).unwrap_or(0);
        let depth_bid_10bps = self.calculate_depth(Side::Bid, 10).unwrap_or(0);
        let depth_ask_10bps = self.calculate_depth(Side::Ask, 10).unwrap_or(0);
        
        let imbalance = if total_bid_volume + total_ask_volume > 0 {
            Some(total_bid_volume as f64 / (total_bid_volume + total_ask_volume) as f64)
        } else {
            None
        };

        OrderBookSnapshot {
            timestamp,
            spread: spread_info.map(|(_, _, s)| s),
            best_bid: spread_info.map(|(b, _, _)| b),
            best_ask: spread_info.map(|(_, a, _)| a),
            mid_price,
            bids,
            asks,
            total_bid_volume,
            total_ask_volume,
            update_counts: self.upd_count.clone(),
            vwap_bid,
            vwap_ask,
            depth_bid_5bps,
            depth_ask_5bps,
            depth_bid_10bps,
            depth_ask_10bps,
            imbalance,
        }
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

use std::collections::BTreeMap;

pub const PRICE_PRECISION: usize = 5;
pub const SIZE_PRECISION: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Default)]
pub struct OrderBook {
    bids: BTreeMap<i32, i32>, // price -> size
    asks: BTreeMap<i32, i32>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn add_level(&mut self, side: Side, price: i32, size: i32) -> Option<i32> {
        if price <= 0 {
            panic!("price must be positive");
        }

        match side {
            Side::Bid => {
                if size <= 0 {
                    return self.bids.remove(&price);
                }
                self.bids.insert(price, size)
            }
            Side::Ask => {
                if size <= 0 {
                    return self.asks.remove(&price);
                }
                self.asks.insert(price, size)
            }
        }
    }

    pub fn best(&self, side: Side) -> Option<(i32, i32)> {
        match side {
            Side::Bid => self.bids.iter().rev().next().map(|(&p, &s)| (p, s)),
            Side::Ask => self.asks.iter().next().map(|(&p, &s)| (p, s)),
        }
    }

    pub fn levels(&self, side: Side) -> Vec<(i32, i32)> {
        match side {
            Side::Bid => self.bids.iter().rev().map(|(&p, &s)| (p, s)).collect(),
            Side::Ask => self.asks.iter().map(|(&p, &s)| (p, s)).collect(),
        }
    }

    pub fn size_at(&self, side: Side, price: i32) -> Option<i32> {
        match side {
            Side::Bid => self.bids.get(&price).copied(),
            Side::Ask => self.asks.get(&price).copied(),
        }
    }

    pub fn print(&self) {
        println!("ASKS:");
        for (p, q) in self.asks.iter().take(10).rev() {
            println!("{} -> {}", p, q);
        }
        println!("");
        println!("BIDS:");
        for (p, q) in self.bids.iter().rev().take(10) {
            println!("{} -> {}", p, q);
        }
        println!("");
        if self.bids.len() > 0 && self.asks.len() > 0 {
            println!(
                "SPREAD: {}",
                self.best(Side::Ask).unwrap().0 as i32
                - self.best(Side::Bid).unwrap().0 as i32
            );
        }
        println!("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_get_levels_bid() {
        let mut ob = OrderBook::new();
        // add bid levels
        assert_eq!(ob.add_level(Side::Bid, 100, 10), None);
        assert_eq!(ob.add_level(Side::Bid, 101, 5), None);
        // best should be 101
        let best = ob.best(Side::Bid).unwrap();
        assert_eq!(best, (101, 5));

        // check ordering
        let levels = ob.levels(Side::Bid);
        assert_eq!(levels, vec![(101, 5), (100, 10)]);

        // update existing level returns previous size
        assert_eq!(ob.add_level(Side::Bid, 100, 15), Some(10));
        assert_eq!(ob.size_at(Side::Bid, 100), Some(15));
    }

    #[test]
    fn add_and_get_levels_ask() {
        let mut ob = OrderBook::new();
        // add ask levels
        assert_eq!(ob.add_level(Side::Ask, 200, 7), None);
        assert_eq!(ob.add_level(Side::Ask, 199, 3), None);
        // best (lowest) should be 199
        let best = ob.best(Side::Ask).unwrap();
        assert_eq!(best, (199, 3));

        // check ordering
        let levels = ob.levels(Side::Ask);
        assert_eq!(levels, vec![(199, 3), (200, 7)]);

        // update existing level returns previous size
        assert_eq!(ob.add_level(Side::Ask, 200, 0), Some(7)); // removing the 200 level
        assert_eq!(ob.size_at(Side::Ask, 200), None);
    }

    #[test]
    fn remove_level_with_zero_size() {
        let mut ob = OrderBook::new();
        ob.add_level(Side::Bid, 50, 12);
        assert_eq!(ob.size_at(Side::Bid, 50), Some(12));
        // remove
        assert_eq!(ob.add_level(Side::Bid, 50, 0), Some(12));
        assert_eq!(ob.size_at(Side::Bid, 50), None);
    }

    #[test]
    #[should_panic(expected = "price must be positive")]
    fn invalid_price_panics() {
        let mut ob = OrderBook::new();
        ob.add_level(Side::Ask, 0, 5);
    }

    #[test]
    fn mixed_sides_independence() {
        let mut ob = OrderBook::new();
        ob.add_level(Side::Bid, 100, 1);
        ob.add_level(Side::Ask, 1000, 2);
        assert_eq!(ob.best(Side::Bid), Some((100, 1)));
        assert_eq!(ob.best(Side::Ask), Some((1000, 2)));
    }
}

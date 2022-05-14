use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug)]
pub(crate) struct Tick {
    pub(crate) exchange: Exchange,
    pub(crate) bids: Vec<Bid>,
    pub(crate) asks: Vec<Ask>,
}

pub(crate) trait ToTick {
    fn maybe_to_tick(&self) -> Option<Tick>;
}

#[derive(Debug)]
pub(crate) enum Exchange {
    Bitstamp,
    Binance,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub(crate) struct Bid {
    price: Decimal,
    amount: Decimal
}

impl Bid {
    pub(crate) fn new(price: Decimal, amount: Decimal) -> Bid {
        Bid{price, amount}
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub(crate) struct Ask {
    price: Decimal,
    amount: Decimal
}

impl Ask {
    pub(crate) fn new(price: Decimal, amount: Decimal) -> Ask {
        Ask{price, amount}
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct MainOrderBook {
    bitstamp: OrderBook,
    binance: OrderBook,
}

impl MainOrderBook {
    pub(crate) fn new() -> MainOrderBook {
        MainOrderBook {
            bitstamp: OrderBook::new(),
            binance: OrderBook::new(),
        }
    }

    /// Extracts the bids and asks from the `Tick`, then adds into its corresponding
    /// orderbook of the exchange.
    pub(crate) fn add(&mut self, t: Tick) {
        let bids = t.bids.iter()
            .map(|b| (b.price, b.amount))
            .collect::<BTreeMap<Decimal, Decimal>>();
        let asks = t.asks.iter()
            .map(|b| (b.price, b.amount))
            .collect::<BTreeMap<Decimal, Decimal>>();

        match t.exchange {
            Exchange::Bitstamp => {
                self.bitstamp.bids = bids;
                self.bitstamp.asks = asks;
            }
            Exchange::Binance => {
                self.binance.bids = bids;
                self.binance.asks = asks;
            }
        }
    }

    /// Returns a new `OrderBook` containing the merge bids and asks from both orderbooks.
    pub(crate) fn merged(&self) -> View {
        let mut bids = self.bitstamp.bids.clone();
        bids.merge(self.binance.bids.clone());
        let mut asks = self.bitstamp.asks.clone();
        asks.merge(self.binance.asks.clone());

        let bids = bids.iter()
            .map(|(k, v)| Bid::new(k.clone(), v.clone()))
            .rev()
            .take(10)
            .collect::<Vec<Bid>>();

        let asks = asks.iter()
            .map(|(k, v)| Ask::new(k.clone(), v.clone()))
            .take(10)
            .collect::<Vec<Ask>>();

        let spread = match (bids.first(), asks.first()) {
            (Some(b), Some(a)) => a.price - b.price,
            (_, _) => dec!(0)
        };

        View{ spread, bids, asks }
    }
}

#[derive(Debug, PartialEq)]
struct OrderBook {
    pub(crate) bids: BTreeMap<Decimal, Decimal>,
    pub(crate) asks: BTreeMap<Decimal, Decimal>,
}

impl OrderBook {
    fn new() -> OrderBook {
        OrderBook{
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }
}

trait Merge {
    fn merge(
        &mut self,
        other: BTreeMap<Decimal, Decimal>,
    );

    fn merge_and_keep(
        &mut self,
        other: BTreeMap<Decimal, Decimal>,
        index: usize,
    );
}

impl Merge for BTreeMap<Decimal, Decimal> {
    /// Same as `BTreeMap::extend` but increments the value instead of replacing it
    /// in case of a duplicate key.
    fn merge(&mut self, other: BTreeMap<Decimal, Decimal>) {
        other.into_iter().for_each(move |(k, v)| {
            match self.get_mut(&k) {
                None => { self.insert(k, v); },
                Some(x) => *x += v, // increment instead of replace
            }
        });
    }

    /// Merges two `BTreeMap`. Returns everything before the given index.
    fn merge_and_keep(&mut self, other: BTreeMap<Decimal, Decimal>, i: usize) {
        self.merge(other);
        if self.len() > i {
            let key = self.keys().collect::<Vec<&Decimal>>()[i].clone();
            self.split_off(&key);
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct View {
    spread: Decimal,
    bids: Vec<Bid>,
    asks: Vec<Ask>,
}

pub(crate) fn channel() -> (UnboundedSender<Tick>, UnboundedReceiver<Tick>) {
    futures::channel::mpsc::unbounded()
}

#[cfg(test)]
mod test {
    use crate::orderbook::*;
    use rust_decimal_macros::dec;
    use std::collections::BTreeMap;

    #[test]
    fn should_add_bitstamp_tick_to_empty() {
        /*
         * Given
         */
        let mut book = MainOrderBook::new();
        let t = Tick{
            exchange: Exchange::Bitstamp,
            bids: vec![
                Bid::new(dec!(0.07358322), dec!(0.46500000)),
                Bid::new(dec!(0.07357954), dec!(8.50000000)),
                Bid::new(dec!(0.07357942), dec!(0.46500000)),
                Bid::new(dec!(0.07357869), dec!(16.31857550)),
                Bid::new(dec!(0.07357533), dec!(2.17483368)),
                Bid::new(dec!(0.07354592), dec!(10.22442936)),
                Bid::new(dec!(0.07354227), dec!(4.34696532)),
                Bid::new(dec!(0.07352810), dec!(20.01159075)),
                Bid::new(dec!(0.07350019), dec!(21.73733228)),
                Bid::new(dec!(0.07348180), dec!(1.85000000)),
            ],
            asks: vec![
                Ask::new(dec!(0.07366569), dec!(0.46500000)),
                Ask::new(dec!(0.07368584), dec!(16.30832712)),
                Ask::new(dec!(0.07371456), dec!(2.17501178)),
                Ask::new(dec!(0.07373077), dec!(4.35024244)),
                Ask::new(dec!(0.07373618), dec!(8.50000000)),
                Ask::new(dec!(0.07374400), dec!(1.85000000)),
                Ask::new(dec!(0.07375536), dec!(11.31202728)),
                Ask::new(dec!(0.07375625), dec!(6.96131361)),
                Ask::new(dec!(0.07375736), dec!(0.00275804)),
                Ask::new(dec!(0.07377938), dec!(0.00275807)),
            ]
        };

        /*
         * When
         */
        book.add(t);

        /*
         * Then
         */
        assert_eq!(book, MainOrderBook {
            bitstamp: OrderBook{
                bids: BTreeMap::from([
                    (dec!(0.07358322), dec!(0.46500000)),
                    (dec!(0.07357954), dec!(8.50000000)),
                    (dec!(0.07357942), dec!(0.46500000)),
                    (dec!(0.07357869), dec!(16.31857550)),
                    (dec!(0.07357533), dec!(2.17483368)),
                    (dec!(0.07354592), dec!(10.22442936)),
                    (dec!(0.07354227), dec!(4.34696532)),
                    (dec!(0.07352810), dec!(20.01159075)),
                    (dec!(0.07350019), dec!(21.73733228)),
                    (dec!(0.07348180), dec!(1.85000000)),
                ]),
                asks: BTreeMap::from([
                    (dec!(0.07366569), dec!(0.46500000)),
                    (dec!(0.07368584), dec!(16.30832712)),
                    (dec!(0.07371456), dec!(2.17501178)),
                    (dec!(0.07373077), dec!(4.35024244)),
                    (dec!(0.07373618), dec!(8.50000000)),
                    (dec!(0.07374400), dec!(1.85000000)),
                    (dec!(0.07375536), dec!(11.31202728)),
                    (dec!(0.07375625), dec!(6.96131361)),
                    (dec!(0.07375736), dec!(0.00275804)),
                    (dec!(0.07377938), dec!(0.00275807)),
                ]),
            },
            binance: OrderBook::new(),
        });
    }

    #[test]
    fn should_merge() {
        /*
         * Given
         */
        let mut book = MainOrderBook::new();
        let t1 = Tick{
            exchange: Exchange::Bitstamp,
            bids: vec![
                Bid::new(dec!(10), dec!(1)),
                Bid::new(dec!(9), dec!(1)),
                Bid::new(dec!(8), dec!(1)),
                Bid::new(dec!(7), dec!(1)),
                Bid::new(dec!(6), dec!(1)),
                Bid::new(dec!(5), dec!(1)),
                Bid::new(dec!(4), dec!(1)),
                Bid::new(dec!(3), dec!(1)),
                Bid::new(dec!(2), dec!(1)),
                Bid::new(dec!(1), dec!(1)),
            ],
            asks: vec![
                Ask::new(dec!(11), dec!(1)),
                Ask::new(dec!(12), dec!(1)),
                Ask::new(dec!(13), dec!(1)),
                Ask::new(dec!(14), dec!(1)),
                Ask::new(dec!(15), dec!(1)),
                Ask::new(dec!(16), dec!(1)),
                Ask::new(dec!(17), dec!(1)),
                Ask::new(dec!(18), dec!(1)),
                Ask::new(dec!(19), dec!(1)),
                Ask::new(dec!(20), dec!(1)),
            ]
        };
        let t2 = Tick{
            exchange: Exchange::Binance,
            bids: vec![
                Bid::new(dec!(10.5), dec!(2)),
                Bid::new(dec!(9.5), dec!(2)),
                Bid::new(dec!(8.5), dec!(2)),
                Bid::new(dec!(7.5), dec!(2)),
                Bid::new(dec!(6.5), dec!(2)),
                Bid::new(dec!(5.5), dec!(2)),
                Bid::new(dec!(4.5), dec!(2)),
                Bid::new(dec!(3.5), dec!(2)),
                Bid::new(dec!(2.5), dec!(2)),
                Bid::new(dec!(1.5), dec!(2)),
            ],
            asks: vec![
                Ask::new(dec!(11.5), dec!(2)),
                Ask::new(dec!(12.5), dec!(2)),
                Ask::new(dec!(13.5), dec!(2)),
                Ask::new(dec!(14.5), dec!(2)),
                Ask::new(dec!(15.5), dec!(2)),
                Ask::new(dec!(16.5), dec!(2)),
                Ask::new(dec!(17.5), dec!(2)),
                Ask::new(dec!(18.5), dec!(2)),
                Ask::new(dec!(19.5), dec!(2)),
                Ask::new(dec!(20.5), dec!(2)),
            ]
        };
        book.add(t1);
        book.add(t2);

        /*
         * When
         */
        let merged = book.merged();

        /*
         * Then
         */
        assert_eq!(merged, View{
            spread: dec!(0.5),
            bids:vec![
                Bid::new(dec!(10.5), dec!(2)),
                Bid::new(dec!(10), dec!(1)),
                Bid::new(dec!(9.5), dec!(2)),
                Bid::new(dec!(9), dec!(1)),
                Bid::new(dec!(8.5), dec!(2)),
                Bid::new(dec!(8), dec!(1)),
                Bid::new(dec!(7.5), dec!(2)),
                Bid::new(dec!(7), dec!(1)),
                Bid::new(dec!(6.5), dec!(2)),
                Bid::new(dec!(6), dec!(1)),
            ],
            asks: vec![
                Ask::new(dec!(11), dec!(1)),
                Ask::new(dec!(11.5), dec!(2)),
                Ask::new(dec!(12), dec!(1)),
                Ask::new(dec!(12.5), dec!(2)),
                Ask::new(dec!(13), dec!(1)),
                Ask::new(dec!(13.5), dec!(2)),
                Ask::new(dec!(14), dec!(1)),
                Ask::new(dec!(14.5), dec!(2)),
                Ask::new(dec!(15), dec!(1)),
                Ask::new(dec!(15.5), dec!(2)),
            ],
        });
    }

}
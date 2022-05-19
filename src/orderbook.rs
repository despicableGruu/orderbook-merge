use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug)]
pub(crate) struct InTick {
    pub(crate) exchange: Exchange,
    pub(crate) bids: Vec<Level>,
    pub(crate) asks: Vec<Level>,
}

pub(crate) trait ToTick {
    fn maybe_to_tick(&self) -> Option<InTick>;
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct OutTick {
    pub(crate) spread: Decimal,
    pub(crate) bids: Vec<Level>,
    pub(crate) asks: Vec<Level>,
}

impl OutTick {
    pub(crate) fn new() -> OutTick {
        OutTick {
            spread: Default::default(),
            bids: vec![],
            asks: vec![],
        }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum Exchange {
    Bitstamp,
    Binance,
}

impl ToString for Exchange {
    fn to_string(&self) -> String {
        match self {
            Exchange::Bitstamp => "bitstamp".to_string(),
            Exchange::Binance => "binance".to_string(),
        }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct Level {
    pub(crate) price: Decimal,
    pub(crate) amount: Decimal,
    pub(crate) exchange: Exchange,
}

impl Level {
    pub(crate) fn new(price: Decimal, amount: Decimal, exchange: Exchange) -> Level {
        Level{price, amount, exchange}
    }
}

pub(crate) trait ToLevel {
    fn to_level(&self) -> Level;
}

pub(crate) trait ToLevels {
    fn to_levels(&self, depth: usize) -> Vec<Level>;
}

impl<T> ToLevels for Vec<T>
    where T: ToLevel + Clone
{
    fn to_levels(&self, depth: usize) -> Vec<Level> {
        let levels = match self.len() > depth {
            true => self.split_at(depth).0.to_vec(), // only keep 10
            false => self.clone(),
        };

        levels.into_iter()
            .map(|l| l.to_level())
            .collect()
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Exchanges {
    bitstamp: OrderDepths,
    binance: OrderDepths,
}

impl Exchanges {
    pub(crate) fn new() -> Exchanges {
        Exchanges {
            bitstamp: OrderDepths::new(),
            binance: OrderDepths::new(),
        }
    }

    /// Extracts the bids and asks from the `InTick`, then adds into its corresponding
    /// orderbook of the exchange.
    pub(crate) fn update(&mut self, t: InTick) {
        match t.exchange {
            Exchange::Bitstamp => {
                self.bitstamp.bids = t.bids;
                self.bitstamp.asks = t.asks;
            },
            Exchange::Binance => {
                self.binance.bids = t.bids;
                self.binance.asks = t.asks;
            },
        }
    }

    /// Returns a new `OutTick` containing the merge bids and asks from both orderbooks.
    pub(crate) fn to_tick(&self) -> OutTick {
        let bids: Vec<Level> =
            Self::merge(self.bitstamp.bids.clone(), self.binance.bids.clone())
                .into_iter()
                .rev()
                .take(10)
                .collect();

        let asks: Vec<Level> =
            Self::merge(self.bitstamp.asks.clone(), self.binance.asks.clone())
                .into_iter()
                .take(10)
                .collect();

        let spread = match (bids.first(), asks.first()) {
            (Some(b), Some(a)) => a.price - b.price,
            (_, _) => dec!(0),
        };

        OutTick { spread, bids, asks }
    }

    fn merge(first: Vec<Level>, second: Vec<Level>) -> Vec<Level> {
        let mut levels: Vec<Level> =
            first.into_iter()
                .chain(second)
                .collect();
        levels.sort_unstable();
        levels
    }
}

#[derive(Debug, PartialEq)]
struct OrderDepths {
    bids: Vec<Level>,
    asks: Vec<Level>,
}

impl OrderDepths {
    fn new() -> OrderDepths {
        OrderDepths {
            bids: vec![],
            asks: vec![],
        }
    }
}

#[cfg(test)]
mod test {
    use crate::orderbook::*;
    use rust_decimal_macros::dec;

    #[test]
    fn should_add_bitstamp_tick_to_empty() {
        /*
         * Given
         */
        let mut exchanges = Exchanges::new();
        let t = InTick {
            exchange: Exchange::Bitstamp,
            bids: vec![
                Level::new(dec!(0.07358322), dec!(0.46500000), Exchange::Bitstamp),
                Level::new(dec!(0.07357954), dec!(8.50000000), Exchange::Bitstamp),
                Level::new(dec!(0.07357942), dec!(0.46500000), Exchange::Bitstamp),
                Level::new(dec!(0.07357869), dec!(16.31857550), Exchange::Bitstamp),
                Level::new(dec!(0.07357533), dec!(2.17483368), Exchange::Bitstamp),
                Level::new(dec!(0.07354592), dec!(10.22442936), Exchange::Bitstamp),
                Level::new(dec!(0.07354227), dec!(4.34696532), Exchange::Bitstamp),
                Level::new(dec!(0.07352810), dec!(20.01159075), Exchange::Bitstamp),
                Level::new(dec!(0.07350019), dec!(21.73733228), Exchange::Bitstamp),
                Level::new(dec!(0.07348180), dec!(1.85000000), Exchange::Bitstamp),
            ],
            asks: vec![
                Level::new(dec!(0.07366569), dec!(0.46500000), Exchange::Bitstamp),
                Level::new(dec!(0.07368584), dec!(16.30832712), Exchange::Bitstamp),
                Level::new(dec!(0.07371456), dec!(2.17501178), Exchange::Bitstamp),
                Level::new(dec!(0.07373077), dec!(4.35024244), Exchange::Bitstamp),
                Level::new(dec!(0.07373618), dec!(8.50000000), Exchange::Bitstamp),
                Level::new(dec!(0.07374400), dec!(1.85000000), Exchange::Bitstamp),
                Level::new(dec!(0.07375536), dec!(11.31202728), Exchange::Bitstamp),
                Level::new(dec!(0.07375625), dec!(6.96131361), Exchange::Bitstamp),
                Level::new(dec!(0.07375736), dec!(0.00275804), Exchange::Bitstamp),
                Level::new(dec!(0.07377938), dec!(0.00275807), Exchange::Bitstamp),
            ],
        };

        /*
         * When
         */
        exchanges.update(t);

        /*
         * Then
         */
        assert_eq!(exchanges, Exchanges {
            bitstamp: OrderDepths {
                bids: vec![
                    Level::new(dec!(0.07358322), dec!(0.46500000), Exchange::Bitstamp),
                    Level::new(dec!(0.07357954), dec!(8.50000000), Exchange::Bitstamp),
                    Level::new(dec!(0.07357942), dec!(0.46500000), Exchange::Bitstamp),
                    Level::new(dec!(0.07357869), dec!(16.31857550), Exchange::Bitstamp),
                    Level::new(dec!(0.07357533), dec!(2.17483368), Exchange::Bitstamp),
                    Level::new(dec!(0.07354592), dec!(10.22442936), Exchange::Bitstamp),
                    Level::new(dec!(0.07354227), dec!(4.34696532), Exchange::Bitstamp),
                    Level::new(dec!(0.07352810), dec!(20.01159075), Exchange::Bitstamp),
                    Level::new(dec!(0.07350019), dec!(21.73733228), Exchange::Bitstamp),
                    Level::new(dec!(0.07348180), dec!(1.85000000), Exchange::Bitstamp),
                ],
                asks: vec![
                    Level::new(dec!(0.07366569), dec!(0.46500000), Exchange::Bitstamp),
                    Level::new(dec!(0.07368584), dec!(16.30832712), Exchange::Bitstamp),
                    Level::new(dec!(0.07371456), dec!(2.17501178), Exchange::Bitstamp),
                    Level::new(dec!(0.07373077), dec!(4.35024244), Exchange::Bitstamp),
                    Level::new(dec!(0.07373618), dec!(8.50000000), Exchange::Bitstamp),
                    Level::new(dec!(0.07374400), dec!(1.85000000), Exchange::Bitstamp),
                    Level::new(dec!(0.07375536), dec!(11.31202728), Exchange::Bitstamp),
                    Level::new(dec!(0.07375625), dec!(6.96131361), Exchange::Bitstamp),
                    Level::new(dec!(0.07375736), dec!(0.00275804), Exchange::Bitstamp),
                    Level::new(dec!(0.07377938), dec!(0.00275807), Exchange::Bitstamp),
                ],
            },
            binance: OrderDepths::new(),
        });
    }

    #[test]
    fn should_merge() {
        /*
         * Given
         */
        let mut exchanges = Exchanges::new();
        let t1 = InTick {
            exchange: Exchange::Bitstamp,
            bids: vec![
                Level::new(dec!(10), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(9), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(8), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(7), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(6), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(5), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(4), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(3), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(2), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(1), dec!(1), Exchange::Bitstamp),
            ],
            asks: vec![
                Level::new(dec!(11), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(12), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(13), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(14), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(15), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(16), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(17), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(18), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(19), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(20), dec!(1), Exchange::Bitstamp),
            ],
        };
        let t2 = InTick {
            exchange: Exchange::Binance,
            bids: vec![
                Level::new(dec!(10.5), dec!(2), Exchange::Binance),
                Level::new(dec!(9.5), dec!(2), Exchange::Binance),
                Level::new(dec!(8.5), dec!(2), Exchange::Binance),
                Level::new(dec!(7.5), dec!(2), Exchange::Binance),
                Level::new(dec!(6.5), dec!(2), Exchange::Binance),
                Level::new(dec!(5.5), dec!(2), Exchange::Binance),
                Level::new(dec!(4.5), dec!(2), Exchange::Binance),
                Level::new(dec!(3.5), dec!(2), Exchange::Binance),
                Level::new(dec!(2.5), dec!(2), Exchange::Binance),
                Level::new(dec!(1.5), dec!(2), Exchange::Binance),
            ],
            asks: vec![
                Level::new(dec!(11.5), dec!(2), Exchange::Binance),
                Level::new(dec!(12.5), dec!(2), Exchange::Binance),
                Level::new(dec!(13.5), dec!(2), Exchange::Binance),
                Level::new(dec!(14.5), dec!(2), Exchange::Binance),
                Level::new(dec!(15.5), dec!(2), Exchange::Binance),
                Level::new(dec!(16.5), dec!(2), Exchange::Binance),
                Level::new(dec!(17.5), dec!(2), Exchange::Binance),
                Level::new(dec!(18.5), dec!(2), Exchange::Binance),
                Level::new(dec!(19.5), dec!(2), Exchange::Binance),
                Level::new(dec!(20.5), dec!(2), Exchange::Binance),
            ],
        };
        exchanges.update(t1);
        exchanges.update(t2);

        /*
         * When
         */
        let out_tick = exchanges.to_tick();

        /*
         * Then
         */
        assert_eq!(out_tick, OutTick {
            spread: dec!(0.5),
            bids:vec![
                Level::new(dec!(10.5), dec!(2), Exchange::Binance),
                Level::new(dec!(10), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(9.5), dec!(2), Exchange::Binance),
                Level::new(dec!(9), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(8.5), dec!(2), Exchange::Binance),
                Level::new(dec!(8), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(7.5), dec!(2), Exchange::Binance),
                Level::new(dec!(7), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(6.5), dec!(2), Exchange::Binance),
                Level::new(dec!(6), dec!(1), Exchange::Bitstamp),
            ],
            asks: vec![
                Level::new(dec!(11), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(11.5), dec!(2), Exchange::Binance),
                Level::new(dec!(12), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(12.5), dec!(2), Exchange::Binance),
                Level::new(dec!(13), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(13.5), dec!(2), Exchange::Binance),
                Level::new(dec!(14), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(14.5), dec!(2), Exchange::Binance),
                Level::new(dec!(15), dec!(1), Exchange::Bitstamp),
                Level::new(dec!(15.5), dec!(2), Exchange::Binance),
            ],
        });
    }

}
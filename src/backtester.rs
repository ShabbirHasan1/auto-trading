use std::borrow::BorrowMut;

use crate::*;

/// 回测器。
pub struct Backtester<T> {
    exchange: T,
    config: Config,
}

impl<T> Backtester<T>
where
    T: Exchange,
{
    /// 构造回测器。
    ///
    /// * `exchange` 交易所。
    /// * `config` 交易配置。
    pub fn new(exchange: T, config: Config) -> Self {
        Self { exchange, config }
    }

    /// 开始回测。
    ///
    /// * `strategy` 策略。
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `k_level` k 线的时间级别。
    /// * `strategy_level` 策略的时间级别，必须大于等于 k 线的时间级别。
    /// * `range` 获取这个时间范围之内的数据，单位毫秒，0 表示获取所有数据，a..b 表示获取 a 到 b 范围的数据。
    /// * `return` 回测结果。
    pub async fn start<F, S, I>(
        &self,
        mut strategy: F,
        product: S,
        k_level: Level,
        strategy_level: Level,
        range: I,
    ) -> anyhow::Result<Vec<Position>>
    where
        F: FnMut(&Context),
        S: AsRef<str>,
        I: Into<TimeRange>,
    {
        let product = product.as_ref();

        if (strategy_level as i32) < (k_level as i32) {
            anyhow::bail!("product: {}: strategy level < k level", product);
        }

        let min_size = self.exchange.get_min_size(product).await?;
        let min_notional = self.exchange.get_min_notional(product).await?;
        let range = range.into();
        let k = get_k_range(&self.exchange, product, k_level, range).await?;
        let time_list = k.iter().map(|v| v.time).collect::<Vec<_>>();
        let open = k.iter().map(|v| v.open).collect::<Vec<_>>();
        let high = k.iter().map(|v| v.high).collect::<Vec<_>>();
        let low = k.iter().map(|v| v.low).collect::<Vec<_>>();
        let close = k.iter().map(|v| v.close).collect::<Vec<_>>();

        let mut me = std::cell::RefCell::new(MatchEngine::new(self.config));

        me.get_mut().product(product, min_size, min_notional);

        for index in (0..time_list.len()).rev().into_iter() {
            let time = time_list[index];
            let open = Source::new(&open[index..]);
            let high = Source::new(&high[index..]);
            let low = Source::new(&low[index..]);
            let close = Source::new(&close[index..]);

            me.borrow_mut().ready(
                product,
                K {
                    time,
                    open: open[0],
                    high: high[0],
                    low: low[0],
                    close: close[0],
                },
            );

            let mut me_ref = me.borrow_mut();

            let cx = Context {
                product,
                min_size,
                min_notional,
                level: strategy_level,
                time,
                open,
                high,
                low,
                close,
                order: &(|a, b, c, d, e, f, g, h| {
                    me.borrow_mut().order(product, a, b, c, d, e, f, g, h)
                }),
                cancel: &(|a| me.borrow_mut().cancel(a)),
                position: &(|| me_ref.position(product)),
            };

            if index == time_list.len() - 1 {
                strategy(&cx);
            } else {
                let millis = match strategy_level {
                    Level::Minute1 => 1000 * 60,
                    Level::Minute3 => 1000 * 60 * 3,
                    Level::Minute5 => 1000 * 60 * 5,
                    Level::Minute15 => 1000 * 60 * 15,
                    Level::Minute30 => 1000 * 60 * 30,
                    Level::Hour1 => 1000 * 60 * 60,
                    Level::Hour2 => 1000 * 60 * 60 * 2,
                    Level::Hour4 => 1000 * 60 * 60 * 4,
                    Level::Hour6 => 1000 * 60 * 60 * 6,
                    Level::Hour12 => 1000 * 60 * 60 * 12,
                    Level::Day1 => 1000 * 60 * 60 * 24,
                    Level::Day3 => 1000 * 60 * 60 * 24 * 3,
                    Level::Week1 => 1000 * 60 * 60 * 24 * 7,
                    Level::Month1 => {
                        // 获取当前时间戳与月初时间戳的差值
                        let now = chrono::Utc::now();
                        std::time::SystemTime::now()
                            .duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64
                            - (chrono::TimeZone::with_ymd_and_hms(
                                &chrono::Utc,
                                chrono::Datelike::year(&now),
                                chrono::Datelike::month(&now),
                                1,
                                0,
                                0,
                                0,
                            )
                            .unwrap()
                            .timestamp_millis() as u64)
                    }
                };

                // 获取指定时间戳日内开始的时间戳
                let day_start = &chrono::NaiveDateTime::from_timestamp_millis(time as i64)
                    .unwrap()
                    .date()
                    .and_hms_opt(0, 0, 0)
                    .unwrap();

                if (time - day_start.timestamp_millis() as u64) % millis == 0 {
                    strategy(&cx);
                }

                me_ref.borrow_mut().update();
            }
        }

        Ok(me.get_mut().history().clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[tokio::test]
    async fn test_1() {
        let k = include_str!("../tests/BTC-USDT-SWAP-1m.json");

        let exchange = LocalExchange::new().push(
            "BTC-USDT-SWAP",
            Level::Minute1,
            serde_json::from_str::<Vec<K>>(k).unwrap(),
            0.01,
            0.0,
        );

        let config = Config::new()
            .initial_margin(1000.0)
            .quantity(Unit::Contract(1))
            .margin(Unit::Quantity(10.0))
            .lever(100)
            .open_fee(0.0002)
            .close_fee(0.0005)
            .maintenance(0.004);

        let backtester = Backtester::new(exchange, config);

        let strategy = |cx: &Context| {
            println!(
                "{} {} {} {} {} {} {}",
                cx.time,
                time_to_string(cx.time),
                cx.open,
                cx.high,
                cx.close,
                cx.low,
                cci(cx.close, 20)
            );
        };

        let result = backtester
            .start(strategy, "BTC-USDT-SWAP", Level::Minute1, Level::Hour4, 0)
            .await;

        println!("{:#?}", result);
    }

    #[tokio::test]
    async fn test_2() {
        let k = include_str!("../tests/BTC-USDT-SWAP-4h.json");

        let exchange = LocalExchange::new().push(
            "BTC-USDT-SWAP",
            Level::Hour4,
            serde_json::from_str::<Vec<K>>(k).unwrap(),
            0.01,
            0.0,
        );

        let config = Config::new()
            .initial_margin(1000.0)
            .quantity(Unit::Contract(1))
            .margin(Unit::Quantity(10.0))
            .lever(100)
            .open_fee(0.0002)
            .close_fee(0.0005)
            .maintenance(0.004);

        let backtester = Backtester::new(exchange, config);

        let strategy = |cx: &Context| {
            println!(
                "{} {} {} {} {} {} {}",
                cx.time,
                time_to_string(cx.time),
                cx.open,
                cx.high,
                cx.close,
                cx.low,
                cci(cx.close, 20)
            );
        };

        let result = backtester
            .start(strategy, "BTC-USDT-SWAP", Level::Hour4, Level::Hour4, 0)
            .await;

        println!("{:#?}", result);
    }
}

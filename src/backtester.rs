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
    /// * `strategy_level` 策略的时间级别，即调用策略函数的时间周期。
    /// * `range` 获取这个时间范围之内的数据，单位毫秒，0 表示获取所有数据，a..b 表示获取 a 到 b 范围的数据。
    /// * `return` 回测结果。
    pub async fn start<F, S, I>(
        &self,
        strategy: F,
        product: S,
        strategy_level: Level,
        range: I,
    ) -> anyhow::Result<Vec<Position>>
    where
        F: FnMut(&Context),
        S: AsRef<str>,
        I: Into<TimeRange>,
    {
        self.start_convert(strategy, product, strategy_level, strategy_level, range)
            .await
    }

    /// 开始回测。
    /// 从交易所获取 `k_level` 时间级别的 k 线数据，然后调用 [`util::k_convert`] 转换到 `strategy_level` 时间级别使用。
    /// 如果 `k_level` 等于 `strategy_level`，则不会发生转换。
    ///
    /// * `strategy` 策略。
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `k_level` k 线的时间级别，撮合引擎会以 k 线的时间级别来处理盈亏，强平，委托。
    /// * `strategy_level` 策略的时间级别，即调用策略函数的时间周期，必须大于等于 k 线的时间级别。
    /// * `range` 获取这个时间范围之内的数据，单位毫秒，0 表示获取所有数据，a..b 表示获取 a 到 b 范围的数据。
    /// * `return` 回测结果。
    pub async fn start_convert<F, S, I>(
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
        let min_size = self.exchange.get_min_size(product).await?;
        let min_notional = self.exchange.get_min_notional(product).await?;
        let range = range.into();
        let k_list = get_k_range(&self.exchange, product, k_level, range).await?;
        let strategy_k_list = if k_level == strategy_level {
            k_list.as_slice()
        } else {
            k_convert(&k_list, strategy_level).leak()
        };
        let open = strategy_k_list.iter().map(|v| v.open).collect::<Vec<_>>();
        let high = strategy_k_list.iter().map(|v| v.high).collect::<Vec<_>>();
        let low = strategy_k_list.iter().map(|v| v.low).collect::<Vec<_>>();
        let close = strategy_k_list.iter().map(|v| v.close).collect::<Vec<_>>();
        let mut me = MatchEngine::new(self.config);
        let mut k_index = k_list.len() - 1;

        me.product(product, min_size, min_notional);

        'a: for (strategy_index, v) in strategy_k_list.iter().enumerate().rev() {
            loop {
                me.ready(
                    product,
                    K {
                        time: k_list[k_index].time,
                        open: k_list[k_index].open,
                        high: k_list[k_index].high,
                        low: k_list[k_index].low,
                        close: k_list[k_index].close,
                    },
                );

                if k_list[k_index].time == v.time {
                    let time = strategy_k_list[strategy_index].time;
                    let open = Source::new(&open[strategy_index..]);
                    let high = Source::new(&high[strategy_index..]);
                    let low = Source::new(&low[strategy_index..]);
                    let close = Source::new(&close[strategy_index..]);
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
                        me: &mut me as *mut MatchEngine,
                    };

                    strategy(&cx);
                    me.update();
                    if k_index == 0 {
                        break 'a;
                    }
                    k_index -= 1;
                    break;
                }

                me.update();
                if k_index == 0 {
                    break;
                }
                k_index -= 1;
            }
        }

        Ok(me.history().clone())
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
                "{} {} {} {} {} {}",
                cx.time,
                time_to_string(cx.time),
                cx.open,
                cx.high,
                cx.low,
                cx.close,
            );
        };

        let result = backtester
            .start_convert(strategy, "BTC-USDT-SWAP", Level::Minute1, Level::Week1, 0)
            .await;

        println!("{:#?}", result);
    }

    #[tokio::test]
    async fn test_2() {
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
                "{} {} {} {} {} {}",
                cx.time,
                time_to_string(cx.time),
                cx.open,
                cx.high,
                cx.low,
                cx.close,
            );
        };

        let result = backtester
            .start_convert(strategy, "BTC-USDT-SWAP", Level::Minute1, Level::Hour4, 0)
            .await;

        println!("{:#?}", result);
    }

    #[tokio::test]
    async fn test_3() {
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
                "{} {} {} {} {} {}",
                cx.time,
                time_to_string(cx.time),
                cx.open,
                cx.high,
                cx.low,
                cx.close,
            );
        };

        let result = backtester
            .start_convert(strategy, "BTC-USDT-SWAP", Level::Hour4, Level::Hour4, 0)
            .await;

        println!("{:#?}", result);
    }

    #[tokio::test]
    async fn test_4() {
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
                "{} {} {} {} {} {}",
                cx.time,
                time_to_string(cx.time),
                cx.open,
                cx.high,
                cx.low,
                cx.close,
            );
        };

        let result = backtester
            .start_convert(strategy, "BTC-USDT-SWAP", Level::Hour4, Level::Hour12, 0)
            .await;

        println!("{:#?}", result);
    }
}

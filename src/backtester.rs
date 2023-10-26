use crate::*;

struct IndexIter<'a> {
    k: &'a [K],
    strategy_k: &'a [K],
    k_index: usize,
    strategy_index: usize,
}

impl<'a> IndexIter<'a> {
    fn new(k: &'a [K], strategy_k: &'a [K]) -> Self {
        Self {
            k,
            strategy_k,
            k_index: k.len(),
            strategy_index: strategy_k.len(),
        }
    }
}

impl<'a> Iterator for IndexIter<'a> {
    type Item = (usize, usize);

    /// 获取策略 k 线放大化后的起始和结尾的下标。
    ///
    /// ```
    /// k = [1000, 900, 800, 700, 600, 500, 400, 300, 200, 100]
    /// strategy = [1000, 700, 400, 100]
    /// Some((7, 3))
    /// Some((4, 2))
    /// Some((1, 1))
    /// None
    /// ```
    fn next(&mut self) -> Option<Self::Item> {
        match self.strategy_k[..self.strategy_index] {
            [.., start, _] => {
                self.k_index = self.k[..self.k_index]
                    .iter()
                    .rposition(|v| v.time >= start.time)?;
                self.strategy_index -= 1;
                (self.k_index + 1 < self.k.len()).then_some((self.strategy_index, self.k_index + 1))
            }
            _ => None,
        }
    }
}

struct Scanner<'a> {
    iter: IndexIter<'a>,
    last: Option<(usize, usize)>,
}

impl<'a> Scanner<'a> {
    fn new(k: &'a [K], strategy_k: &'a [K]) -> Self {
        Self {
            iter: IndexIter::new(k, strategy_k),
            last: None,
        }
    }

    fn get(&mut self) -> Option<(usize, usize)> {
        self.last = self.last.or_else(|| self.iter.next());
        self.last
    }

    fn next(&mut self) {
        self.last = self.iter.next()
    }
}

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
    /// * `strategy_level` 策略的时间级别，即调用策略的时间周期。
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
        F: FnMut(&mut Context),
        S: AsRef<str>,
        I: Into<TimeRange>,
    {
        self.start_amplifier(strategy, product, strategy_level, strategy_level, range)
            .await
    }

    /// 开始回测。
    /// 从交易所获取 `k_level` 和 strategy_level 时间级别的 k 线数据。
    ///
    /// * `strategy` 策略。
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `k_level` k 线的时间级别，撮合引擎会以 k 线的时间级别来处理盈亏，强平，委托。
    /// * `strategy_level` 策略的时间级别，即调用策略的时间周期。
    /// * `range` 获取这个时间范围之内的数据，单位毫秒，0 表示获取所有数据，a..b 表示获取 a 到 b 范围的数据。
    /// * `return` 回测结果。
    pub async fn start_amplifier<F, S, I>(
        &self,
        mut strategy: F,
        product: S,
        k_level: Level,
        strategy_level: Level,
        range: I,
    ) -> anyhow::Result<Vec<Position>>
    where
        F: FnMut(&mut Context),
        S: AsRef<str>,
        I: Into<TimeRange>,
    {
        struct TradingImpl {
            me: MatchEngine,
        }

        impl TradingImpl {
            fn new(config: Config) -> Self {
                Self {
                    me: MatchEngine::new(config),
                }
            }
        }

        impl Trading for TradingImpl {
            fn order(
                &mut self,
                product: &str,
                side: Side,
                price: f64,
                quantity: Unit,
                margin: Unit,
                stop_profit_condition: Unit,
                stop_loss_condition: Unit,
                stop_profit: Unit,
                stop_loss: Unit,
            ) -> anyhow::Result<u64> {
                self.me.order(
                    product,
                    side,
                    price,
                    quantity,
                    margin,
                    stop_profit_condition,
                    stop_loss_condition,
                    stop_profit,
                    stop_loss,
                )
            }

            fn cancel(&mut self, id: u64) -> bool {
                self.me.cancel(id)
            }

            fn balance(&self) -> f64 {
                self.me.balance()
            }

            fn delegate(&self, id: u64) -> Option<DelegateState> {
                self.me.delegate(id)
            }

            fn position(&self, product: &str) -> Option<&Position> {
                self.me.position(product)
            }
        }

        anyhow::ensure!(
            (k_level as u32) <= (strategy_level as u32),
            "product: {}: strategy level must be greater than k level",
            product.as_ref(),
        );

        let product = product.as_ref();
        let range = range.into();
        let min_size = self.exchange.get_min_size(product).await?;
        let min_notional = self.exchange.get_min_notional(product).await?;
        let k = get_k_range(&self.exchange, product, k_level, range).await?;
        let strategy_k;

        let strategy_k = if k_level == strategy_level {
            k.as_slice()
        } else {
            strategy_k = get_k_range(&self.exchange, product, strategy_level, range).await?;
            strategy_k.as_slice()
        };

        let open = strategy_k.iter().map(|v| v.open).collect::<Vec<_>>();
        let high = strategy_k.iter().map(|v| v.high).collect::<Vec<_>>();
        let low = strategy_k.iter().map(|v| v.low).collect::<Vec<_>>();
        let close = strategy_k.iter().map(|v| v.close).collect::<Vec<_>>();

        let mut scanner = Scanner::new(&k, &strategy_k);
        let mut ti = TradingImpl::new(self.config);

        ti.me.insert_product(product, min_size, min_notional);

        for index in (0..k.len()).rev() {
            ti.me.ready(
                product,
                K {
                    time: k[index].time,
                    open: k[index].open,
                    high: k[index].high,
                    low: k[index].low,
                    close: k[index].close,
                },
            );

            if let Some((start_index, end_index)) = if k_level == strategy_level {
                Some((index, index))
            } else {
                scanner.get()
            } {
                if index == end_index {
                    let time = strategy_k[start_index].time;
                    let open = Source::new(&open[start_index..]);
                    let high = Source::new(&high[start_index..]);
                    let low = Source::new(&low[start_index..]);
                    let close = Source::new(&close[start_index..]);

                    let mut cx = Context {
                        product,
                        min_size,
                        min_notional,
                        level: strategy_level,
                        time,
                        open,
                        high,
                        low,
                        close,
                        trading: &mut ti,
                    };

                    strategy(&mut cx);

                    scanner.next()
                }
            }

            ti.me.update();
        }

        Ok(ti.me.history().clone())
    }
}

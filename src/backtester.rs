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
    /// * `strategy_level` 策略的时间级别，即调用策略的时间周期，必须大于等于 k 线的时间级别。
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
        F: FnMut(&mut Context),
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
        me.insert_product(product, min_size, min_notional);

        struct IndexIter<'a> {
            k_list: &'a [K],
            strategy_list: &'a [K],
            k_index: usize,
            strategy_index: usize,
        }

        impl<'a> IndexIter<'a> {
            fn new(k_list: &'a [K], strategy_list: &'a [K]) -> Self {
                Self {
                    k_list,
                    strategy_list,
                    k_index: k_list.len(),
                    strategy_index: strategy_list.len(),
                }
            }
        }

        impl<'a> Iterator for IndexIter<'a> {
            type Item = (usize, usize);

            fn next(&mut self) -> Option<Self::Item> {
                // [1000, 900, 800, 700, 600, 500, 400, 300, 200, 100]
                // [1000, 700, 400, 100]
                // Some((7, 3))
                // Some((4, 2))
                // Some((1, 1))
                // None
                if let [.., start, _] = self.strategy_list[..self.strategy_index] {
                    self.k_index = self.k_list[..self.k_index]
                        .iter()
                        .rposition(|v| v.time >= start.time)?;
                    self.strategy_index -= 1;

                    return (self.k_index + 1 < self.k_list.len())
                        .then_some((self.k_index + 1, self.strategy_index));
                }

                None
            }
        }

        let mut index_iter = IndexIter::new(&k_list, strategy_k_list);
        let mut end_index = None;

        for k_index in (0..k_list.len()).rev() {
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

            if let Some((index, strategy_index)) = {
                if k_level == strategy_level {
                    Some((k_index, k_index))
                } else {
                    match end_index {
                        Some(v) => Some(v),
                        None => {
                            end_index = index_iter.next();
                            end_index
                        }
                    }
                }
            } {
                if k_index == index {
                    let time = strategy_k_list[strategy_index].time;
                    let open = Source::new(&open[strategy_index..]);
                    let high = Source::new(&high[strategy_index..]);
                    let low = Source::new(&low[strategy_index..]);
                    let close = Source::new(&close[strategy_index..]);
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
                        me: &mut me as *mut MatchEngine,
                    };

                    strategy(&mut cx);

                    end_index = None;
                }
            }

            me.update();
        }

        Ok(me.history().clone())
    }
}

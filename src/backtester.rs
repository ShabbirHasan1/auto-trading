use crate::*;

pub struct Backtester<T> {
    bourse: T,
    mark_price: bool,
    config: Config,
    strategy: Vec<(String, Box<dyn Fn(&Context)>)>,
}

impl<T> Backtester<T>
where
    T: Bourse,
{
    /// * `bourse` 交易所。
    pub fn new(bourse: T) -> Self {
        Self {
            bourse,
            mark_price: false,
            config: Config::new(),
            strategy: Vec::new(),
        }
    }

    /// 使用标记价格。
    pub fn mark_price(mut self, value: bool) -> Self {
        self.mark_price = value;
        self
    }

    /// 交易配置。
    pub fn config(mut self, value: Config) -> Self {
        self.config = value;
        self
    }

    /// 添加策略。
    pub fn strategy<S>(mut self, value: S) -> Self
    where
        S: Fn(&Context) + 'static,
    {
        self.strategy.push((
            format!("unnamed strategy {}", self.strategy.len()),
            Box::new(value),
        ));
        self
    }

    /// 添加策略。
    ///
    /// * `name` 策略名字。
    pub fn strategy_name<N, S>(mut self, name: N, value: S) -> Self
    where
        N: AsRef<str>,
        S: Fn(&Context) + 'static,
    {
        self.strategy
            .push((name.as_ref().to_string(), Box::new(value)));
        self
    }

    /// 开始回测。
    ///
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `level` 时间级别。
    /// * `time` 获取这个时间之前的数据，0 表示获取所有数据。
    /// * `return` 回测结果，如果 [`Config`] 的某些参数未设置，且策略依赖这些参数，将返回错误。
    pub async fn start<S, I>(
        &self,
        product: S,
        level: Level,
        time: I,
    ) -> anyhow::Result<Vec<Position>>
    where
        S: AsRef<str>,
        I: Into<TimeRange>,
    {
        let product = product.as_ref();
        let k = self.get_k_range(product, level, time.into()).await?;
        let time = k.iter().map(|v| v.time).collect::<Vec<_>>();
        let open = k.iter().map(|v| v.open).collect::<Vec<_>>();
        let high = k.iter().map(|v| v.high).collect::<Vec<_>>();
        let low = k.iter().map(|v| v.low).collect::<Vec<_>>();
        let close = k.iter().map(|v| v.close).collect::<Vec<_>>();

        // 变量表
        let mut variable = std::collections::BTreeMap::new();

        // 订单簿
        let order_book = todo!();

        // 仓位
        let position = todo!();

        for index in (0..time.len()).rev().into_iter() {
            let time = time[index];
            let open = Source::new(&open[index..]);
            let high = Source::new(&high[index..]);
            let low = Source::new(&low[index..]);
            let close = Source::new(&close[index..]);
            fn ff() {}

            for (name, strategy) in self.strategy {
                let order =
                    |side: Side, price: f64, size: Unit, stop_profit: f64, stop_loss: f64| {
                        let inner = || -> anyhow::Result<()> {
                            if price == 0.0 {
                                if size.is_zero() {
                                    self.config
                                        .margin
                                        .ok_or(anyhow::anyhow!("uninitialized: config.margin"))
                                } else {
                                    Ok(size)
                                }?;

                                Position {
                                    product: product.to_string(),
                                    isolated: self.config.isolated,
                                    lever: self.config.lever,
                                    side,
                                    open_price: price,
                                    close_price: f64::NAN,
                                    open_quantity: todo!(),
                                    close_quantity: todo!(),
                                    profit: 0.0,
                                    profit_ratio: 0.0,
                                    open_time: todo!(),
                                    close_time: todo!(),
                                };
                            } else {
                                Position {
                                    product: product.to_string(),
                                    isolated: self.config.isolated,
                                    lever: self.config.lever,
                                    side,
                                    open_price: price,
                                    close_price: f64::NAN,
                                    open_quantity: todo!(),
                                    close_quantity: todo!(),
                                    profit: 0.0,
                                    profit_ratio: 0.0,
                                    open_time: todo!(),
                                    close_time: todo!(),
                                };
                            }

                            todo!()
                        };
                    };
            }

            let cx = Context {
                product,
                level,
                time,
                open,
                high,
                low,
                close,
                variable: &mut variable,
                order: todo!(),
                cancel: todo!(),
                new_context: todo!(),
            };
        }

        todo!()
    }

    /// 开始回测。
    ///
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `level` 时间级别。
    /// * `k` K 线数据。
    /// * `return` 回测结果，如果 [`Config`] 的某些参数未设置，且策略依赖这些参数，将返回错误。
    pub fn start_with_k<S>(&self, product: S, level: Level, k: Vec<K>)
    where
        S: AsRef<str>,
    {
    }

    async fn get_k_range(
        &self,
        product: &str,
        level: Level,
        time: TimeRange,
    ) -> anyhow::Result<Vec<K>> {
        let mut result = Vec::new();

        if time.start == 0 && time.end == 0 {
            let mut time = 0;

            loop {
                let v = self.get_k(product, level, time).await?;

                if let Some(k) = v.last() {
                    time = k.time;
                    result.extend(v);
                } else {
                    break;
                }
            }

            return Ok(result);
        }

        let mut start = time.start;

        loop {
            let v = self.get_k(product, level, start).await?;

            if let Some(k) = v.last() {
                if k.time <= time.end {
                    break;
                }

                start = k.time;
                result.extend(v);
            } else {
                break;
            }
        }

        Ok(result)
    }

    async fn get_k(&self, product: &str, level: Level, time: u64) -> anyhow::Result<Vec<K>> {
        if self.mark_price {
            self.bourse.get_k_mark(product, level, time)
        } else {
            self.bourse.get_k(product, level, time)
        }
        .await
    }
}

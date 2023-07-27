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

        // 单笔最小交易数量
        let min_unit = self.bourse.get_min_unit(product).await?;

        // 余额
        let balance = self.config.initial_margin;

        // 变量表
        let mut variable = std::collections::HashMap::<&'static str, Value>::new();

        // 订单簿
        let order_book = todo!();

        // 仓位
        let position = Vec::<Position>::new();

        for index in (0..time.len()).rev().into_iter() {
            let time = time[index];
            let open = Source::new(&open[index..]);
            let high = Source::new(&high[index..]);
            let low = Source::new(&low[index..]);
            let close = Source::new(&close[index..]);

            Context {
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

            let order =
                |side: Side, price: f64, margin: Unit, stop_profit: Unit, stop_loss: Unit| {
                    let mut price = price;
                    let mut margin = margin;

                    if price == 0.0 {
                        price = close[0];
                    }

                    if margin == 0.0 {
                        margin = self.config.margin;
                    }

                    let margin = match margin {
                        Quantity(v) => v,
                        Proportion(v) => self.config.initial_margin * v,
                    };

                    // 张转换到 USDT
                    let min_unit = min_unit * price;

                    // 可开张数
                    let count = (margin * self.config.lever as f64 / min_unit) as u64;

                    // 持仓量 USDT
                    let open_quantity = price * count as f64;

                    // 持仓量小于一张
                    if open_quantity < min_unit {
                        anyhow::bail!(
                            "open quantity < min unit: {} USDT < {} USDT",
                            margin,
                            min_unit
                        );
                    }

                    // 保证金不足
                    if balance < margin + self.config.fee {
                        anyhow::bail!(
                            "balance < margin + fee: {} USDT < {} USDT",
                            balance,
                            margin + self.config.fee
                        );
                    }

                    match side {
                        Side::BuyLong => {
                            if self.config.position_mode {
                                let mut sum = 0.0;
                                let mut count = 0.0;
                                for i in position {
                                    if i.product == product && i.side == Side::BuyLong {
                                        sum += i.open_price * i.open_quantity;
                                        count += 1.0;
                                    }
                                }
                                sum = (sum + close[0]) / (count + 1.0);

                                // 逐仓
                                if self.config.isolated {}

                                Position {
                                    product: product.to_string(),
                                    isolated: self.config.isolated,
                                    lever: self.config.lever,
                                    side,
                                    margin,
                                    open_price: close[0],
                                    close_price: 0.0,
                                    open_quantity,
                                    liquidation_price: todo!(),
                                    profit: todo!(),
                                    profit_ratio: todo!(),
                                    fee: todo!(),
                                    open_time: todo!(),
                                    close_time: todo!(),
                                    list: todo!(),
                                };
                            } else {
                            }
                        }
                        Side::SellShort => {}
                        Side::SellLong => {}
                        Side::BuySell => {}
                    }

                    anyhow::Ok(())
                };

            for (name, strategy) in self.strategy {}
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

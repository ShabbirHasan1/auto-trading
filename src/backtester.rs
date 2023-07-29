use crate::*;

/// 回测器。
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
    /// 构造回测器。
    ///
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
    /// * `time` 获取这个时间范围之内的数据，0 表示获取所有数据。
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
        // TODO: 如何兼容现货和合约？
        // TODO: 如何实现交割合约和期权合约？
        // TODO: 要不要支持移动止盈止损？
        // TODO: 触发限价委托，不占用保证金，但是官方没有这个接口？？？？？？

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
        let mut balance = self.config.initial_margin;

        // let balance = &mut _balance;

        // 变量表
        let mut variable = std::collections::HashMap::<&'static str, Value>::new();

        // 仓位
        let mut position = Vec::<Position>::new();

        // 订单簿
        let mut order_book = Vec::<Delegate>::new();

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

            let mut order = |side: Side,
                             price: f64,
                             margin: Unit,
                             stop_profit_condition: Unit,
                             stop_loss_condition: Unit,
                             stop_profit: Unit,
                             stop_loss: Unit| {
                // 计算开仓均价
                let avg_open_price =
                    |product: &str, side: Side, open_price: f64, open_quantity: f64| {
                        if let Some((i, v)) = position.iter().enumerate().find(|v| {
                            v.1.product == product
                                && if self.config.position_mode {
                                    v.1.side == side
                                } else {
                                    true
                                }
                        }) {
                            let quantity = v.open_quantity - open_quantity;

                            let side = if quantity == 0.0 {
                                side
                            } else if quantity > 0.0 {
                                Side::BuyLong
                            } else {
                                Side::SellShort
                            };

                            let a = v.open_price * v.open_quantity * side.factor();
                            let b = open_price * open_quantity * side.factor();
                            let open_price = (a + b) / 2.0;

                            return (i, side, open_price, quantity.abs());
                        }

                        (usize::MAX, side, open_price, open_quantity)
                    };

                // TODO: 单向持仓的话，如果多空一起开，会先平掉其中一个仓位
                if self.config.isolated {
                    match side {
                        Side::BuyLong | Side::SellShort => {
                            // 限价或市价
                            let open_price =
                                if price == 0.0 { close[0] } else { price } * self.config.deviation;

                            // 单笔投入的保证金
                            let margin = if margin == 0.0 {
                                self.config.margin
                            } else {
                                margin
                            }
                            .to_quantity(self.config.initial_margin);

                            // 张换算到 USDT
                            let min_unit = min_unit * open_price;

                            // 可开张数
                            let count = (margin * self.config.lever as f64 / min_unit) as u64;

                            // 持仓量 USDT
                            let open_quantity = open_price * count as f64;

                            // 持仓量小于一张
                            anyhow::ensure!(
                                open_quantity < min_unit,
                                "open quantity < min unit: {} USDT < {} USDT",
                                margin,
                                min_unit
                            );

                            let temp = margin + self.config.fee;

                            // 保证金不足
                            anyhow::ensure!(
                                balance < temp,
                                "balance < margin + fee: {} USDT < {} USDT",
                                balance,
                                temp
                            );

                            balance -= temp;

                            let (index, new_side, new_open_price, new_open_quantity) =
                                avg_open_price(product, side, open_price, open_quantity);

                            // 强平价格 = (入场价格 × (1 + 初始保证金率 - 维持保证金率)) ± (追加保证金 / 仓位数量)。
                            // 初始保证金率 = 1 / 杠杆
                            // 维持保证金率 = 0.005
                            // 追加保证金 = 账户余额 - 初始化保证金
                            // 初始化保证金 = 入场价格 / 杠杆
                            let liquidation_price = new_open_price
                                / (1.0 + 1.0 / self.config.lever as f64 - self.config.maintenance)
                                - new_side.factor();

                            if index == usize::MAX {
                                let mut temp = Position {
                                    product: product.to_string(),
                                    isolated: self.config.isolated,
                                    lever: self.config.lever,
                                    side,
                                    margin,
                                    open_price,
                                    close_price: 0.0,
                                    open_quantity,
                                    liquidation_price,
                                    profit: 0.0,
                                    profit_ratio: 0.0,
                                    fee: self.config.fee,
                                    open_time: time,
                                    close_time: 0,
                                    list: Vec::new(),
                                };

                                temp.list.push(ChildPosition {
                                    side,
                                    margin,
                                    price: open_price,
                                    quantity: open_quantity,
                                    profit: 0.0,
                                    profit_ratio: 0.0,
                                    time,
                                });

                                position.push(temp);
                            } else {
                                let position = &mut position[index];
                                position.side = new_side;
                                position.open_price = new_open_price;
                                position.open_quantity = new_open_quantity;
                                position.list.push(ChildPosition {
                                    side,
                                    margin,
                                    price: open_price,
                                    quantity: open_quantity,
                                    profit: 0.0,
                                    profit_ratio: 0.0,
                                    time,
                                })
                            }

                            // Delegate {
                            //     product: product.to_string(),
                            //     isolated: self.config.isolated,
                            //     lever: self.config.lever,
                            //     side,
                            //     price,
                            //     quantity,
                            //     stop_profit_condition: stop_profit_condition.to_quantity(value),
                            //     stop_loss_condition: stop_loss_condition,
                            //     stop_profit: 0,
                            //     stop_loss: 0,
                            // };
                        }
                        Side::SellLong => {
                            let position = position
                                .iter()
                                .find(|v| v.product == product && v.side == Side::SellShort);

                            anyhow::ensure!(
                                position.is_some(),
                                "cannot find the position: {}",
                                product
                            );

                            let position = position.unwrap();

                            let margin = margin.to_quantity(position.open_quantity);

                            // 张换算到 USDT
                            // let min_unit = min_unit * open_price;

                            // // 可开张数
                            // let count = (margin * self.config.lever as f64 / min_unit) as u64;

                            // // 持仓量 USDT
                            // let open_quantity = open_price * count as f64;

                            // // 张换算到 USDT
                            // let min_unit = min_unit * 1;

                            // margin / min_unit;
                        }
                        Side::BuySell => {
                            // TODO: 要考虑现货的平仓
                            let margin = match margin {
                                Quantity(v) => v,
                                Proportion(v) => {
                                    position
                                        .iter()
                                        .filter(|v| v.side == Side::BuyLong)
                                        .map(|v| v.open_quantity)
                                        .sum::<f64>()
                                        * v
                                }
                            };
                        }
                    }
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

use crate::*;

/// 回测器。
pub struct Backtester<T> {
    bourse: T,
    config: Config,
    mark_price: bool,
    other_product: bool,
}

impl<T> Backtester<T>
where
    T: Bourse,
{
    /// 构造回测器。
    ///
    /// * `bourse` 交易所。
    /// * `config` 交易配置。
    pub fn new(bourse: T, config: Config) -> Self {
        Self {
            bourse,
            config,
            mark_price: false,
            other_product: false,
        }
    }

    /// 使用标记价格。
    pub fn mark_price(mut self, value: bool) -> Self {
        self.mark_price = value;
        self
    }

    /// 允许策略下单 `start` 函数参数中 `product` 之外的交易产品。
    pub fn other_product(mut self, value: bool) -> Self {
        self.other_product = value;
        self
    }

    /// 开始回测。
    ///
    /// * `strategy` 策略。
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `strategy_level` 策略的时间级别。
    /// * `backtester_level` 回测器的精度。
    /// * `time` 获取这个时间范围之内的数据，0 表示获取所有数据。
    /// * `return` 回测结果，如果 [`Config`] 的某些参数未设置，且策略依赖这些参数，将返回错误。
    pub async fn start<F, S, I>(
        &self,
        strategy: F,
        product: S,
        strategy_level: Level,
        backtester_level: Level,
        time: I,
    ) -> anyhow::Result<Vec<Position>>
    where
        F: Fn(&Context),
        S: AsRef<str>,
        I: Into<TimeRange>,
    {
        // TODO: 如何兼容现货和合约？
        // TODO: 如何实现交割合约和期权合约？
        // TODO: 要不要支持移动止盈止损？
        // TODO: 触发限价委托，不占用保证金，但是官方没有这个接口？？？？？？

        let product = product.as_ref();

        let k = self
            .get_k_range(product, strategy_level, time.into())
            .await?;

        let time = k.iter().map(|v| v.time).collect::<Vec<_>>();
        let open = k.iter().map(|v| v.open).collect::<Vec<_>>();
        let high = k.iter().map(|v| v.high).collect::<Vec<_>>();
        let low = k.iter().map(|v| v.low).collect::<Vec<_>>();
        let close = k.iter().map(|v| v.close).collect::<Vec<_>>();

        // 单笔最小交易数量
        let min_unit = self.bourse.get_min_unit(product).await?;

        // 变量表
        let mut variable = std::collections::HashMap::<&'static str, Value>::new();

        // 撮合引擎
        let mut me = MatchmakingEngine::new(self.config);

        for index in (0..time.len()).rev().into_iter() {
            let time = time[index];
            let open = Source::new(&open[index..]);
            let high = Source::new(&high[index..]);
            let low = Source::new(&low[index..]);
            let close = Source::new(&close[index..]);

            let mut order = |side,
                             mut price: f64,
                             quantity: Unit,
                             stop_profit_condition: Unit,
                             stop_loss_condition: Unit,
                             stop_profit: Unit,
                             stop_loss: Unit| {
                if price == 0.0 {
                    price = close[0];
                }

                let quantity =
                    quantity.to_quantity(self.config.initial_margin) * self.config.lever as f64;

                me.delegate(Delegate {
                    product: product.to_string(),
                    isolated: self.config.isolated,
                    lever: self.config.lever,
                    side,
                    price,
                    quantity,
                    margin: quantity / self.config.lever as f64,
                    stop_profit_condition: price
                        + stop_profit_condition.to_quantity(price) * side.factor(),
                    stop_loss_condition: price
                        + stop_loss_condition.to_quantity(price) * side.factor(),
                    stop_profit: price + stop_profit.to_quantity(price) * side.factor(),
                    stop_loss: price + stop_loss.to_quantity(price) * side.factor(),
                })
            };

            let cx = Context {
                product,
                level: strategy_level,
                time,
                open,
                high,
                low,
                close,
                variable: &mut variable,
                order: &mut order,
                cancel: todo!(),
                new_context: todo!(),
            };

            strategy(&mut cx);
        }

        Ok(Vec::new())
    }

    pub async fn get_k_range(
        &self,
        product: &str,
        level: Level,
        time: TimeRange,
    ) -> anyhow::Result<Vec<K>> {
        // TODO: 请求太快会返回错误，返回的是请求太快错误的话，继续请求

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

        let mut end = time.end;

        loop {
            let v = self.get_k(product, level, end).await?;

            if let Some(k) = v.last() {
                if k.time <= time.start {
                    break;
                }

                end = k.time;
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

/// 撮合引擎。
pub struct MatchmakingEngine {
    balance: f64,
    config: Config,
    delegate: Vec<Delegate>,
    position: Vec<(Position, Vec<SubDelegate>)>,
    history: Vec<Position>,
}

impl MatchmakingEngine {
    pub fn new(config: Config) -> Self {
        Self {
            balance: config.initial_margin,
            config,
            delegate: Vec::new(),
            position: Vec::new(),
            history: Vec::new(),
        }
    }

    /// 返回订单 id。
    pub fn delegate(&mut self, value: Delegate) -> anyhow::Result<usize> {
        // 检查止盈止损是否有利于仓位
        if value.side == Side::BuyLong {
            if value.stop_profit_condition <= value.price {
                anyhow::bail!(
                    "stop profit condition <= price: {} USDT <= {}",
                    value.stop_profit_condition,
                    value.price
                );
            }

            if value.stop_loss_condition >= value.price {
                anyhow::bail!(
                    "stop profit condition >= price: {} USDT >= {}",
                    value.stop_loss_condition,
                    value.price
                );
            }
        } else {
            if value.stop_profit_condition >= value.price {
                anyhow::bail!(
                    "stop profit condition >= price: {} USDT >= {}",
                    value.stop_profit_condition,
                    value.price
                );
            }

            if value.stop_loss_condition <= value.price {
                anyhow::bail!(
                    "stop profit condition <= price: {} USDT <= {}",
                    value.stop_loss_condition,
                    value.price
                );
            }
        };

        // 检查平仓量是否小于等于持仓量
        if value.side == Side::BuySell && value.side == Side::SellLong {
            if let Some(v) = self
                .position
                .iter()
                .find(|v| v.0.product == value.product)
                .map(|v| &v.0)
            {
                anyhow::ensure!(
                    v.open_quantity <= value.quantity,
                    "close quantity must be less than or equal open quantity"
                );
            } else {
                anyhow::bail!("no position: {}", value.product);
            }
        }

        self.delegate.push(value);

        Ok(self.delegate.len() - 1)
    }

    /// 更新仓位。
    ///
    /// * `pair` 交易产品和价格。
    pub fn update<T>(&mut self, time: u64, pair: T) -> anyhow::Result<()>
    where
        T: AsRef<[(String, f64)]>,
    {
        let pair = pair.as_ref();

        // 处理跟随仓位的委托
        for (position, sub_delegate) in self.position.iter_mut() {
            for price in pair.iter().filter(|v| v.0 == position.product).map(|v| v.1) {
                for i in (0..sub_delegate.len()).rev() {
                    let delegate = &mut sub_delegate[i];
                    if delegate.side == Side::BuySell && price >= delegate.condition
                        || price <= delegate.condition
                    {
                        if delegate.condition == delegate.price {
                            // 限价触发，市价委托
                            let margin = delegate.quantity / position.lever as f64;
                            let profit = (position.open_price - delegate.price) * delegate.quantity;
                            let profit_ratio = profit / margin;
                            self.balance += profit;
                            self.balance -= self.config.fee;
                            position.margin -= margin;
                            position.fee += self.config.fee;
                            position.list.push(SubPosition {
                                side: delegate.side,
                                price: delegate.condition,
                                quantity: delegate.quantity,
                                margin,
                                profit,
                                profit_ratio,
                                time,
                            });
                            sub_delegate.swap_remove(i);
                        } else {
                            // 限价触发，限价委托
                            delegate.condition = delegate.price;
                        }
                    }
                }
            }
        }

        // 处理盈亏
        for (product, price) in pair {
            for i in self.position.iter_mut().filter(|v| v.0.product == *product) {
                i.0.profit = (price - i.0.open_price) * i.0.open_quantity;
                i.0.profit_ratio = i.0.profit / i.0.margin;
            }
        }

        // 处理强平
        'a: for (product, price) in pair {
            for i in (0..self.position.len()).rev() {
                let v = &self.position[i];
                if *product == v.0.product
                    && (v.0.side == Side::BuyLong && *price >= v.0.liquidation_price
                        || *price <= v.0.liquidation_price)
                {
                    if v.0.isolated {
                        let mut v = self.position.swap_remove(i);
                        v.0.close_price = *price;
                        v.0.close_time = time;
                        self.history.push(v.0);
                    } else {
                        for (product, price) in pair {
                            for i in (0..self.position.len()).rev() {
                                let v = &self.position[i];
                                if *product == v.0.product {
                                    let mut v = self.position.swap_remove(i);
                                    v.0.close_price = *price;
                                    v.0.close_time = time;
                                    self.history.push(v.0);
                                }
                            }
                        }
                        break 'a;
                    }
                }
            }
        }

        // 处理委托
        for i in (0..self.delegate.len()).rev() {
            let delegate = &self.delegate[i];
            for price in pair.iter().filter(|v| v.0 == delegate.product).map(|v| v.1) {
                if delegate.side == Side::BuySell || delegate.side == Side::SellLong {
                    // 平仓委托
                    let sub_delegate = self
                        .position
                        .iter_mut()
                        .find(|v| v.0.product == delegate.product)
                        .map(|v| &mut v.1)
                        .unwrap();

                    sub_delegate.push(SubDelegate {
                        side: delegate.side,
                        quantity: delegate.quantity,
                        condition: delegate.price,
                        price,
                    });

                    return Ok(());
                }

                if delegate.stop_profit_condition == 0.0 && delegate.stop_profit == 0.0 {
                    anyhow::bail!(
                        "because stop profit condition is zero, so stop profit must be zero"
                    );
                }

                if delegate.stop_loss_condition == 0.0 && delegate.stop_loss == 0.0 {
                    anyhow::bail!("because stop loss condition is zero, so stop loss must be zero")
                }

                if delegate.isolated {
                    let margin = delegate.quantity / delegate.lever as f64;

                    // 查找现有仓位
                    let position = self.position.iter_mut().find(|v| {
                        v.0.product == delegate.product
                            && if self.config.position_mode {
                                v.0.side == delegate.side
                            } else {
                                true
                            }
                    });

                    // 计算开仓均价
                    let (new_side, new_price, new_quantity) = match position.as_ref() {
                        Some(v) => {
                            let quantity = v.0.open_quantity - delegate.quantity;

                            let side = if quantity == 0.0 {
                                delegate.side
                            } else if quantity > 0.0 {
                                Side::BuyLong
                            } else {
                                Side::SellShort
                            };

                            let a = v.0.open_price * v.0.open_quantity * side.factor();
                            let b = delegate.price * delegate.quantity * delegate.side.factor();
                            let open_price = (a + b) / 2.0;

                            (side, open_price, quantity.abs())
                        }
                        None => (delegate.side, delegate.price, delegate.quantity),
                    };

                    // 强平价格 = (入场价格 × (1 + 初始保证金率 - 维持保证金率)) ± (追加保证金 / 仓位数量)
                    // 初始保证金率 = 1 / 杠杆
                    // 维持保证金率 = 0.005
                    // 追加保证金 = 账户余额 - 初始化保证金
                    // 初始保证金 = 入场价格 / 杠杆
                    let liquidation_price = new_price
                        / (1.0 + 1.0 / self.config.lever as f64 - self.config.maintenance)
                        - new_side.factor();

                    if price <= delegate.price {
                        // 子仓位
                        let cp = SubPosition {
                            side: delegate.side,
                            price: delegate.price,
                            quantity: delegate.quantity,
                            margin,
                            profit: 0.0,
                            profit_ratio: 0.0,
                            time: 0,
                        };

                        // 止盈委托
                        let sp = SubDelegate {
                            side: delegate.side.neg(),
                            quantity: delegate.quantity,
                            condition: delegate.stop_profit_condition,
                            price: delegate.stop_profit,
                        };

                        // 止损委托
                        let sl = SubDelegate {
                            side: delegate.side.neg(),
                            quantity: delegate.quantity,
                            condition: delegate.stop_loss_condition,
                            price: delegate.stop_loss,
                        };

                        match position {
                            Some((position, sub_delegate)) => {
                                // 已经存在仓位
                                position.side = new_side;
                                position.open_price = new_price;
                                position.open_quantity = new_quantity;
                                position.liquidation_price = liquidation_price;
                                position.fee += self.config.fee;
                                position.list.push(cp);
                                sub_delegate.push(sp);
                                sub_delegate.push(sl);
                            }
                            None => {
                                // 新建仓位
                                let mut position = Position {
                                    product: delegate.product.clone(),
                                    isolated: delegate.isolated,
                                    lever: self.config.lever,
                                    side: new_side,
                                    open_price: new_price,
                                    open_quantity: new_quantity,
                                    margin,
                                    liquidation_price,
                                    close_price: 0.0,
                                    profit: 0.0,
                                    profit_ratio: 0.0,
                                    fee: self.config.fee,
                                    open_time: time,
                                    close_time: 0,
                                    list: Vec::new(),
                                };

                                position.list.push(cp);
                                let mut sub_delegate = Vec::new();
                                sub_delegate.push(sp);
                                sub_delegate.push(sl);
                                self.position.push((position, sub_delegate));
                            }
                        };
                    }
                }
            }
        }

        Ok(())
    }
}

struct SubDelegate {
    /// 方向。
    side: Side,

    /// 持仓量。
    quantity: f64,

    /// 条件。
    condition: f64,

    /// 价格。
    price: f64,
}

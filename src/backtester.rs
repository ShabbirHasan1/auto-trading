use crate::*;

/// 回测器。
pub struct Backtester<T> {
    bourse: T,
    config: Config,
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
            other_product: false,
        }
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
    /// * `time` 获取这个时间范围之内的数据，单位毫秒，0 表示获取所有数据，a..b 表示获取 a 到 b 范围的数据。
    /// * `return` 回测结果，如果 [`Config`] 的某些参数未设置，且策略依赖这些参数，将返回错误。
    pub async fn start<F, S, I>(
        &self,
        mut strategy: F,
        product: S,
        strategy_level: Level,
        backtester_level: Level,
        time: I,
    ) -> anyhow::Result<Vec<Position>>
    where
        F: FnMut(&mut Context),
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

        // let sk = self
        //     .get_k_range(product, backtester_level, time.into())
        //     .await?;

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
        let me = std::cell::RefCell::new(MatchmakingEngine::new(self.config));

        for index in (0..time.len()).rev().into_iter() {
            let time = time[index];
            let open = Source::new(&open[index..]);
            let high = Source::new(&high[index..]);
            let low = Source::new(&low[index..]);
            let close = Source::new(&close[index..]);

            let order = |side: Side,
                         mut price: f64,
                         quantity: Unit,
                         stop_profit_condition: Unit,
                         stop_loss_condition: Unit,
                         stop_profit: Unit,
                         stop_loss: Unit| {
                if price == 0.0 {
                    price = close[0];
                }

                let quantity = if quantity == 0.0 {
                    self.config.margin.to_quantity(self.config.initial_margin)
                        * self.config.lever as f64
                } else {
                    quantity.to_quantity(self.config.initial_margin) * self.config.lever as f64
                };

                let unit = price * min_unit;

                if quantity < unit {
                    anyhow::bail!("quantity < min unit: {} < {}", quantity, price * min_unit);
                }

                let quantity = (quantity / unit).floor() * unit;

                let stop_profit_condition = if stop_profit_condition == 0.0 {
                    if self.config.stop_profit == 0.0 {
                        0.0
                    } else {
                        price + self.config.stop_profit.to_quantity(price) * side.factor()
                    }
                } else {
                    price + stop_profit_condition.to_quantity(price) * side.factor()
                };

                let stop_loss_condition = if stop_profit_condition == 0.0 {
                    if self.config.stop_loss == 0.0 {
                        0.0
                    } else {
                        price + self.config.stop_loss.to_quantity(price) * side.factor()
                    }
                } else {
                    stop_loss_condition.to_quantity(price) * side.factor()
                };

                let stop_profit = if stop_profit == 0.0 {
                    0.0
                } else {
                    price + stop_profit.to_quantity(price) * side.factor()
                };

                let stop_loss = if stop_loss == 0.0 {
                    0.0
                } else {
                    price + stop_loss.to_quantity(price) * side.factor()
                };

                let result = me.borrow_mut().delegate(Delegate {
                    product: product.to_string(),
                    isolated: self.config.isolated,
                    lever: self.config.lever,
                    side,
                    price,
                    quantity,
                    margin: quantity / self.config.lever as f64,
                    stop_profit_condition,
                    stop_loss_condition,
                    stop_profit,
                    stop_loss,
                });

                result
            };

            let cancel = |value: u64| me.borrow_mut().cancel(value);

            let mut cx = Context {
                product,
                level: strategy_level,
                time,
                open,
                high,
                low,
                close,
                order: &order,
                cancel: &cancel,
                new_context: &|a, b| todo!(),
            };

            strategy(&mut cx);

            me.borrow_mut()
                .update(time, [(product, close[0], min_unit)])
                .unwrap();
        }

        println!("==> 委托");
        println!("{:#?}", me.borrow().delegate);
        println!("==> 仓位");
        println!("{:#?}", me.borrow().position);
        println!("==> 历史 ");
        println!("{:#?}", me.borrow().history);

        Ok(Vec::new())
    }

    pub async fn get_k_range(
        &self,
        product: &str,
        level: Level,
        time: TimeRange,
    ) -> anyhow::Result<Vec<K>> {
        let mut result = Vec::new();

        if time.start == 0 && time.end == 0 {
            let mut time = 0;

            loop {
                let v = self.bourse.get_k(product, level, time).await?;

                if let Some(k) = v.last() {
                    time = k.time;
                    result.extend(v);
                } else {
                    result.extend(v);
                    break;
                }
            }

            return Ok(result);
        }

        let mut end = time.end;

        if end == u64::MAX - 1 {
            end = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
        }

        loop {
            let v = self.bourse.get_k(product, level, end).await?;

            if let Some(k) = v.last() {
                if k.time < time.start {
                    for i in v {
                        if i.time >= time.start {
                            result.push(i);
                        }
                    }
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
}

/// 撮合引擎。
#[derive(Debug, Clone)]
pub struct MatchmakingEngine {
    balance: f64,
    config: Config,
    delegate: std::collections::HashMap<u64, Delegate>,
    position: Vec<(Position, Vec<SubDelegate>)>,
    history: Vec<Position>,
}

impl MatchmakingEngine {
    pub fn new(config: Config) -> Self {
        Self {
            balance: config.initial_margin,
            config,
            delegate: std::collections::HashMap::new(),
            position: Vec::new(),
            history: Vec::new(),
        }
    }

    /// 委托。
    pub fn delegate(&mut self, mut value: Delegate) -> anyhow::Result<u64> {
        // 检查止盈参数
        if value.side == Side::BuyLong || value.side == Side::SellShort {
            if value.stop_profit_condition == 0.0 && value.stop_profit != 0.0 {
                anyhow::bail!(
                    "the stop profit must be zero, because the stop profit condition is zero"
                );
            }

            if value.stop_loss_condition == 0.0 && value.stop_loss != 0.0 {
                anyhow::bail!("the stop loss must be zero, because the stop loss condition is zero")
            }
        }

        // 检查止盈止损是否有利于仓位
        if value.side == Side::BuyLong {
            if value.stop_profit_condition != 0.0 {
                if value.stop_profit_condition <= value.price {
                    anyhow::bail!(
                        "buy long, but stop profit <= price: {} <= {}",
                        value.stop_profit_condition,
                        value.price
                    );
                }
            }

            if value.stop_loss_condition != 0.0 {
                if value.stop_loss_condition >= value.price {
                    anyhow::bail!(
                        "buy long, but stop loss >= price: {} >= {}",
                        value.stop_loss_condition,
                        value.price
                    );
                }
            }
        } else if value.side == Side::SellShort {
            if value.stop_profit_condition != 0.0 {
                if value.stop_profit_condition >= value.price {
                    anyhow::bail!(
                        "sell short, but stop profit >= price: {} >= {}",
                        value.stop_profit_condition,
                        value.price
                    );
                }
            }

            if value.stop_loss_condition != 0.0 {
                if value.stop_loss_condition <= value.price {
                    anyhow::bail!(
                        "sell short, but stop loss <= price: {} <= {}",
                        value.stop_loss_condition,
                        value.price
                    );
                }
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
                    value.quantity <= v.open_quantity,
                    "close quantity must be less than or equal open quantity"
                );
            } else {
                anyhow::bail!("no position: {}", value.product);
            }
        }

        // 检查余额
        if self.balance <= value.margin + self.config.fee {
            anyhow::bail!(
                "insufficient fund: balance < margin + fee: {} < {} + {}",
                self.balance,
                value.margin,
                self.config.fee
            );
        }

        self.balance -= value.margin + self.config.fee;

        let id = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        self.delegate.insert(id, value);

        Ok(id)
    }

    /// 取消订单。
    pub fn cancel(&mut self, value: u64) {
        self.delegate.remove(&value);
    }

    /// 更新仓位。
    ///
    /// * `pair` 交易产品，价格，面值。
    pub fn update<'a, T>(&mut self, time: u64, pair: T) -> anyhow::Result<()>
    where
        T: AsRef<[(&'a str, f64, f64)]>,
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
                        if delegate.price == 0.0 {
                            // TODO: 注意这里，使用市价还是 condition？
                            // TODO: 注意计算平仓量
                            // TODO: 保证金计算可能不准确，考虑加入 margin
                            // 限价触发，市价委托

                            let margin = delegate.quantity / position.lever as f64;
                            let profit = (price - position.open_price) * delegate.quantity
                                / position.open_price;
                            let profit_ratio = profit / margin;

                            self.balance += profit;
                            self.balance -= self.config.fee;
                            self.balance += margin;

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
                            // TODO: 删除平仓委托
                        } else {
                            // 限价触发，限价委托
                            delegate.condition = delegate.price;
                        }
                    }
                }
            }
        }

        // 处理盈亏
        for (product, price, _) in pair {
            for i in self.position.iter_mut().filter(|v| v.0.product == *product) {
                i.0.profit = (price - i.0.open_price) * i.0.open_quantity / i.0.open_price;
                i.0.profit_ratio = i.0.profit / i.0.margin;
                // 应该卖出的时候才计算盈亏
                for i in i.0.list.iter_mut() {
                    i.profit = (price - i.price) * i.quantity / i.price;
                    i.profit = i.profit / i.margin;
                }
            }
        }

        // 处理强平
        'a: for (product, price, _) in pair {
            for i in (0..self.position.len()).rev() {
                let v = &self.position[i];
                if *product == v.0.product
                    && (v.0.side == Side::BuyLong && *price <= v.0.liquidation_price
                        || v.0.side == Side::SellShort && *price >= v.0.liquidation_price)
                {
                    if v.0.isolated {
                        // TODO: 全仓强平价格是一直变的
                        // TODO: 逐仓强平价格应该是精准的？不是！
                        // TODO: 注意这里，使用市价还是 condition？
                        let mut v = self.position.swap_remove(i);
                        v.0.close_price = *price;
                        v.0.close_time = time;
                        self.history.push(v.0);
                        // TODO: 强平也加入子仓位？
                    } else {
                        for (product, price, _) in pair {
                            for i in (0..self.position.len()).rev() {
                                let v = &self.position[i];
                                if *product == v.0.product {
                                    // TODO: 注意这里，使用市价还是 condition？
                                    let mut v = self.position.swap_remove(i);
                                    v.0.close_price = *price;
                                    v.0.close_time = time;
                                    self.history.push(v.0);
                                    // TODO: 强平也加入子仓位？
                                }
                            }
                        }
                        break 'a;
                    }
                }
            }
        }

        // 处理委托
        self.delegate.retain(|_, delegate| {
            for price in pair.iter().filter(|v| v.0 == delegate.product).map(|v| v.1) {
                // 平仓委托
                if delegate.side == Side::BuySell || delegate.side == Side::SellLong {
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

                    return false;
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
                            let quantity = v.0.open_quantity * v.0.side.factor()
                                + delegate.quantity * delegate.side.factor();

                            let side = if quantity == 0.0 {
                                delegate.side
                            } else if quantity > 0.0 {
                                Side::BuyLong
                            } else {
                                Side::SellShort
                            };

                            let a = v.0.open_price * v.0.open_quantity * side.factor();
                            let b = delegate.price * delegate.quantity * delegate.side.factor();
                            let open_price = (a + b) / (v.0.open_quantity + delegate.quantity);

                            (side, open_price, quantity.abs())
                        }
                        None => (delegate.side, delegate.price, delegate.quantity.abs()),
                    };

                    // 做多强平价格 = (入场价格 × (1 - 初始保证金率 + 维持保证金率)) - (追加保证金 / 仓位数量)
                    // 做空强平价格 = (入场价格 × (1 + 初始保证金率 - 维持保证金率)) + (追加保证金 / 仓位数量)
                    // 初始保证金率 = 1 / 杠杆
                    // 维持保证金率 = 0.005
                    // 追加保证金 = 账户余额 - 初始化保证金
                    // 初始保证金 = 入场价格 / 杠杆
                    let imr = 1.0 / self.config.lever as f64;
                    let mmr = self.config.maintenance;
                    let liquidation_price =
                        new_price * (1.0 + imr * -new_side.factor() + mmr * new_side.factor());

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
                                position.margin += margin;
                                position.liquidation_price = liquidation_price;
                                position.fee += self.config.fee;
                                position.list.push(cp);

                                if sp.condition != 0.0 {
                                    sub_delegate.push(sp);
                                }

                                if sl.condition != 0.0 {
                                    sub_delegate.push(sl);
                                }
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

                                if sp.condition != 0.0 {
                                    sub_delegate.push(sp);
                                }

                                if sl.condition != 0.0 {
                                    sub_delegate.push(sl);
                                }

                                self.position.push((position, sub_delegate));
                            }
                        };
                    }
                    return false;
                }

                return true;
            }
            return true;
        });

        // for price in pair.iter().filter(|v| v.0 == delegate.product).map(|v| v.1) {
        //     // 平仓委托
        //     if delegate.side == Side::BuySell || delegate.side == Side::SellLong {
        //         let sub_delegate = self
        //             .position
        //             .iter_mut()
        //             .find(|v| v.0.product == delegate.product)
        //             .map(|v| &mut v.1)
        //             .unwrap();

        //         sub_delegate.push(SubDelegate {
        //             side: delegate.side,
        //             quantity: delegate.quantity,
        //             condition: delegate.price,
        //             price,
        //         });

        //         return Ok(());
        //     }

        //     if delegate.isolated {
        //         let margin = delegate.quantity / delegate.lever as f64;

        //         // 查找现有仓位
        //         let position = self.position.iter_mut().find(|v| {
        //             v.0.product == delegate.product
        //                 && if self.config.position_mode {
        //                     v.0.side == delegate.side
        //                 } else {
        //                     true
        //                 }
        //         });

        //         // 计算开仓均价
        //         let (new_side, new_price, new_quantity) = match position.as_ref() {
        //             Some(v) => {
        //                 let quantity = v.0.open_quantity - delegate.quantity;

        //                 let side = if quantity == 0.0 {
        //                     delegate.side
        //                 } else if quantity > 0.0 {
        //                     Side::BuyLong
        //                 } else {
        //                     Side::SellShort
        //                 };

        //                 let a = v.0.open_price * v.0.open_quantity * side.factor();
        //                 let b = delegate.price * delegate.quantity * delegate.side.factor();
        //                 let open_price = (a + b) / (v.0.open_quantity + delegate.quantity);

        //                 (side, open_price, quantity.abs())
        //             }
        //             None => (delegate.side, delegate.price, delegate.quantity),
        //         };

        //         // 强平价格 = (入场价格 × (1 + 初始保证金率 - 维持保证金率)) ± (追加保证金 / 仓位数量)
        //         // 初始保证金率 = 1 / 杠杆
        //         // 维持保证金率 = 0.005
        //         // 追加保证金 = 账户余额 - 初始化保证金
        //         // 初始保证金 = 入场价格 / 杠杆
        //         let liquidation_price = new_price
        //             / (1.0 + 1.0 / self.config.lever as f64 - self.config.maintenance)
        //             - new_side.factor();

        //         if price <= delegate.price {
        //             // 子仓位
        //             let cp = SubPosition {
        //                 side: delegate.side,
        //                 price: delegate.price,
        //                 quantity: delegate.quantity,
        //                 margin,
        //                 profit: 0.0,
        //                 profit_ratio: 0.0,
        //                 time: 0,
        //             };

        //             // 止盈委托
        //             let sp = SubDelegate {
        //                 side: delegate.side.neg(),
        //                 quantity: delegate.quantity,
        //                 condition: delegate.stop_profit_condition,
        //                 price: delegate.stop_profit,
        //             };

        //             // 止损委托
        //             let sl = SubDelegate {
        //                 side: delegate.side.neg(),
        //                 quantity: delegate.quantity,
        //                 condition: delegate.stop_loss_condition,
        //                 price: delegate.stop_loss,
        //             };

        //             match position {
        //                 Some((position, sub_delegate)) => {
        //                     // 已经存在仓位
        //                     position.side = new_side;
        //                     position.open_price = new_price;
        //                     position.open_quantity = new_quantity;
        //                     position.liquidation_price = liquidation_price;
        //                     position.fee += self.config.fee;
        //                     position.list.push(cp);
        //                     sub_delegate.push(sp);
        //                     sub_delegate.push(sl);
        //                 }
        //                 None => {
        //                     // 新建仓位
        //                     let mut position = Position {
        //                         product: delegate.product.clone(),
        //                         isolated: delegate.isolated,
        //                         lever: self.config.lever,
        //                         side: new_side,
        //                         open_price: new_price,
        //                         open_quantity: new_quantity,
        //                         margin,
        //                         liquidation_price,
        //                         close_price: 0.0,
        //                         profit: 0.0,
        //                         profit_ratio: 0.0,
        //                         fee: self.config.fee,
        //                         open_time: time,
        //                         close_time: 0,
        //                         list: Vec::new(),
        //                     };

        //                     position.list.push(cp);
        //                     let mut sub_delegate = Vec::new();
        //                     sub_delegate.push(sp);
        //                     sub_delegate.push(sl);
        //                     self.position.push((position, sub_delegate));
        //                 }
        //             };
        //         }
        //     }

        //     self.delegate.remove(id);
        // }

        Ok(())
    }
}

#[derive(Debug, Clone)]
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

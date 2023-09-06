use crate::*;

/// 回测器。
pub struct Backtester<T> {
    exchange: T,
    config: Config,
    other_product: bool,
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
        Self {
            exchange,
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
    /// * `level` 时间级别。
    /// * `range` 获取这个时间范围之内的数据，单位毫秒，0 表示获取所有数据，a..b 表示获取 a 到 b 范围的数据。
    /// * `return` 回测结果，如果 [`Config`] 的某些参数未设置，且策略依赖这些参数，将返回错误。
    pub async fn start<F, S, I>(
        &self,
        mut strategy: F,
        product: S,
        level: Level,
        range: I,
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
        // TODO: 考虑删除 neg
        // TODO: position_mode 未经测试
        // TODO: 最大持仓量和 fee 计算冗余

        let product = product.as_ref();

        let range = range.into();

        let k = get_k_range(&self.exchange, product, level, range).await?;

        let time = k.iter().map(|v| v.time).collect::<Vec<_>>();

        let open = k.iter().map(|v| v.open).collect::<Vec<_>>();

        let high = k.iter().map(|v| v.high).collect::<Vec<_>>();

        let low = k.iter().map(|v| v.low).collect::<Vec<_>>();

        let close = k.iter().map(|v| v.close).collect::<Vec<_>>();

        let unit = self.exchange.get_unit(product).await?;

        let me = std::cell::RefCell::new(MatchmakingEngine::new(self.config));

        me.borrow_mut().product(product, unit);

        for index in (0..time.len()).rev().into_iter() {
            let time = time[index];
            let open = Source::new(&open[index..]);
            let high = Source::new(&high[index..]);
            let low = Source::new(&low[index..]);
            let close = Source::new(&close[index..]);

            me.borrow_mut()
                .ready(product, time, high[0], low[0], close[0]);

            let mut cx = Context {
                product,
                level,
                unit,
                time,
                open,
                high,
                low,
                close,
                order: &|side,
                         price,
                         quantity,
                         stop_profit_condition,
                         stop_loss_condition,
                         stop_profit,
                         stop_loss| {
                    me.borrow_mut().order(
                        product,
                        side,
                        price,
                        quantity,
                        stop_profit_condition,
                        stop_loss_condition,
                        stop_profit,
                        stop_loss,
                    )
                },
                cancel: &|value| {
                    me.borrow_mut().cancel(value);
                },
                position: &|product| {
                    if product.is_empty() {
                        me.borrow().position.get(0).map(|v| v.0.clone())
                    } else {
                        me.borrow()
                            .position
                            .iter()
                            .find(|v| v.0.product == product)
                            .map(|v| v.0.clone())
                    }
                },
                new_context: &|product: &str, level: Level| {
                    todo!("貌似无解，策略闭包要不要改成异步的？")
                },
            };

            strategy(&mut cx);

            me.borrow_mut().update();
        }

        Ok(me.into_inner().history)
    }
}

/// 撮合引擎。
#[derive(Debug, Clone)]
pub struct MatchmakingEngine {
    /// 余额。
    balance: f64,

    /// 交易配置。
    config: Config,

    /// 订单 id。
    id: u64,

    /// 产品，相关数据。
    product: std::collections::BTreeMap<String, PriceData>,

    /// 仓位，开仓委托。
    delegate: std::collections::BTreeMap<u64, Delegate>,

    /// 仓位，平仓委托。
    position: Vec<(Position, Vec<SubDelegate>)>,

    /// 历史仓位。
    history: Vec<Position>,
}

impl MatchmakingEngine {
    pub fn new(value: Config) -> Self {
        Self {
            balance: value.initial_margin,
            config: value,
            id: 0,
            product: std::collections::BTreeMap::new(),
            delegate: std::collections::BTreeMap::new(),
            position: Vec::new(),
            history: Vec::new(),
        }
    }

    /// 插入产品。
    ///
    /// * `product` 交易产品。
    /// * `unit` 面值，1 张 = 价格 * 面值。
    pub fn product<S>(&mut self, product: S, unit: f64)
    where
        S: AsRef<str>,
    {
        let product = product.as_ref();
        self.product.insert(
            product.to_string(),
            PriceData {
                unit,
                time: 0,
                high: 0.0,
                low: 0.0,
                close: 0.0,
            },
        );
    }

    /// 准备。
    /// 在调用委托之前，需要准备。
    /// 在准备之前，需要插入产品。
    ///
    /// * `product` 交易产品。
    /// * `time` 时间。
    /// * `high` 最高价。
    /// * `low` 最低价。
    /// * `close` 收盘价。
    pub fn ready<S>(&mut self, product: S, time: u64, high: f64, low: f64, close: f64)
    where
        S: AsRef<str>,
    {
        let product = product.as_ref();
        let temp = self
            .product
            .get_mut(product)
            .expect(&format!("no product: {}", product));
        temp.time = time;
        temp.high = high;
        temp.low = low;
        temp.close = close;
    }

    /// 下单。
    /// 如果做多限价大于当前价格，那么价格大于等于限价的时候才会成交。
    /// 如果做空限价小于当前价格，那么价格小于等于限价的时候才会成交。
    /// 如果策略在价格到达 [`Config`] 止盈止损目标位之前没有平仓操作，则仓位会进行平仓操作。
    /// 平仓不会导致仓位反向开单，平仓数量只能小于等于现有持仓数量。
    /// 如果在进行平仓操作后，现有的限价平仓委托的平仓量小于持仓量，则该委托将被撤销。
    ///
    /// * `product` 交易产品。
    /// * `side` 订单方向。
    /// * `price` 委托价格，0 表示市价，其他表示限价。
    /// * `quantity` 委托数量，如果是开仓，则 0 表示最小下单数量，[`Unit::Proportion`] 表示占用初始保证金的比例，如果是平仓，则 0 表示全部仓位，[`Unit::Proportion`] 表示占用仓位的比例。
    /// * `stop_profit_condition` 止盈触发价格，0 表示不设置，且 `stop_profit` 无效。
    /// * `stop_loss_condition` 止损触发价格，0 表示不设置，且 `stop_loss` 无效。
    /// * `stop_profit` 止盈委托价格，0 表示市价，其他表示限价。
    /// * `stop_loss` 止损委托格，0 表示市价，其他表示限价。
    /// * `return` 订单 id。
    pub fn order<S>(
        &mut self,
        product: S,
        side: Side,
        price: f64,
        quantity: Unit,
        stop_profit_condition: Unit,
        stop_loss_condition: Unit,
        stop_profit: Unit,
        stop_loss: Unit,
    ) -> anyhow::Result<u64>
    where
        S: AsRef<str>,
    {
        let product = product.as_ref();

        let PriceData { unit, close, .. } = self
            .product
            .get(product)
            .ok_or(anyhow::anyhow!("no product: {}", product))?
            .to_owned();

        if side == Side::BuyLong || side == Side::SellShort {
            let price = if price == 0.0 { close } else { price };

            // 最小下单价值
            let min_unit = unit * price;

            // 策略仓位价值
            let mut strategy_quantity =
                quantity.to_quantity(self.config.initial_margin * self.config.lever as f64);

            if strategy_quantity == 0.0 {
                strategy_quantity = min_unit;
            }

            // 仓位价值
            let quantity = if self.config.margin == 0.0 {
                if self.config.quantity == 0.0 {
                    (strategy_quantity / min_unit).floor() * min_unit
                } else {
                    anyhow::bail!(
                        "product {}: config margin cannot be zero, because config quantity is not zero",
                        product
                    );
                }
            } else {
                if self.config.quantity == 0.0 {
                    (strategy_quantity / min_unit).floor() * min_unit
                } else {
                    match self.config.margin {
                        Unit::Quantity(v) => {
                            (v * self.config.quantity * self.config.lever as f64 / min_unit).floor()
                                * min_unit
                        }
                        Unit::Proportion(v) => {
                            (strategy_quantity * (v - self.config.quantity) / min_unit).floor()
                                * min_unit
                        }
                    }
                }
            };

            // 开仓价值不能小于最小下单价值
            if quantity < min_unit {
                anyhow::bail!(
                    "product {}: open quantity < min unit: {} < {}",
                    product,
                    quantity,
                    min_unit
                );
            }

            let margin = if self.config.margin == 0.0 {
                // 仓位维持保证金
                quantity / self.config.lever as f64
            } else {
                // 实际投入的保证金
                match self.config.margin {
                    Unit::Quantity(v) => v,
                    Unit::Proportion(v) => strategy_quantity * v / self.config.lever as f64,
                }
            };

            // 手续费
            let fee = quantity * self.config.open_fee;

            // 检查余额
            if self.balance < margin + fee {
                anyhow::bail!(
                    "product {}: insufficient fund: balance < position margin + fee: {} < {} + {}",
                    product,
                    self.balance,
                    margin,
                    fee
                );
            }

            let temp = self.position.iter().map(|v| v.0.margin).sum::<f64>() + margin;

            // 检查最大投入的保证金数量
            if self.config.max_margin != 0.0 && temp > self.config.max_margin {
                anyhow::bail!(
                    "product {}: position margin > max margin: {} > {}",
                    product,
                    temp,
                    self.config.max_margin,
                );
            }

            // 检查止盈止损参数
            if stop_profit_condition == 0.0 && stop_profit != 0.0 {
                anyhow::bail!(
                    "product {}: stop profit must be zero, because stop profit condition is zero",
                    product
                );
            }

            if stop_loss_condition == 0.0 && stop_loss != 0.0 {
                anyhow::bail!(
                    "product {}: stop loss must be zero, because stop loss condition is zero",
                    product
                )
            }

            // 检查止盈止损是否有利于仓位
            let stop_profit_condition = if stop_profit_condition == 0.0 {
                0.0
            } else {
                match stop_profit_condition {
                    Unit::Quantity(v) => v,
                    Unit::Proportion(v) => price + price * v * side.factor(),
                }
            };

            let stop_loss_condition = if stop_loss_condition == 0.0 {
                0.0
            } else {
                match stop_loss_condition {
                    Unit::Quantity(v) => v,
                    Unit::Proportion(v) => price + price * v * side.neg().factor(),
                }
            };

            let stop_profit = if stop_profit == 0.0 {
                0.0
            } else {
                match stop_profit {
                    Unit::Quantity(v) => v,
                    Unit::Proportion(v) => price + price * v * side.factor(),
                }
            };

            let stop_loss = if stop_loss == 0.0 {
                0.0
            } else {
                match stop_loss {
                    Unit::Quantity(v) => v,
                    Unit::Proportion(v) => price + price * v * side.neg().factor(),
                }
            };

            if side == Side::BuyLong {
                if stop_profit_condition != 0.0 {
                    if stop_profit_condition <= price {
                        anyhow::bail!(
                            "product {}: buy long, but stop profit <= price: {} <= {}",
                            product,
                            stop_profit_condition,
                            price
                        );
                    }
                }

                if stop_loss_condition != 0.0 {
                    if stop_loss_condition >= price {
                        anyhow::bail!(
                            "product {}: buy long, but stop loss >= price: {} >= {}",
                            product,
                            stop_loss_condition,
                            price
                        );
                    }
                }
            } else {
                if stop_profit_condition != 0.0 {
                    if stop_profit_condition >= price {
                        anyhow::bail!(
                            "product {}: sell short, but stop profit >= price: {} >= {}",
                            product,
                            stop_profit_condition,
                            price
                        );
                    }
                }

                if stop_loss_condition != 0.0 {
                    if stop_loss_condition <= price {
                        anyhow::bail!(
                            "product {}: sell short, but stop loss <= price: {} <= {}",
                            product,
                            stop_loss_condition,
                            price
                        );
                    }
                }
            };

            self.balance -= margin + fee;

            self.id += 1;

            self.delegate.insert(
                self.id,
                Delegate {
                    product: product.to_string(),
                    lever: self.config.lever,
                    side,
                    price,
                    quantity,
                    margin,
                    stop_profit_condition,
                    stop_loss_condition,
                    stop_profit,
                    stop_loss,
                },
            );

            return Ok(self.id);
        }

        if let Some(v) = self
            .position
            .iter()
            .find(|v| v.0.product == product)
            .map(|v| &v.0)
        {
            let price = if price == 0.0 { close } else { price };

            // 最小下单价值
            let min_unit = unit * v.open_price;

            // 转换百分比
            let quantity = if quantity == 0.0 {
                v.quantity
            } else {
                (quantity.to_quantity(v.quantity) / min_unit).floor() as f64
            };

            // 平仓数量不能小于最小下单价值
            if quantity < min_unit {
                anyhow::bail!(
                    "product {}: close quantity < min unit: {} < {}",
                    product,
                    quantity,
                    min_unit
                );
            }

            // 平仓量要小于持仓量
            if quantity > v.quantity {
                anyhow::bail!(
                    "product {}: close quantity > open quantity: {} > {}",
                    product,
                    quantity,
                    v.quantity,
                );
            };

            self.id += 1;

            self.delegate.insert(
                self.id,
                Delegate {
                    product: product.to_string(),
                    lever: self.config.lever,
                    side,
                    price,
                    quantity,
                    margin: quantity / self.config.lever as f64,
                    stop_profit_condition: 0.0,
                    stop_loss_condition: 0.0,
                    stop_profit: 0.0,
                    stop_loss: 0.0,
                },
            );

            return Ok(self.id);
        }

        anyhow::bail!("no position: {}", product);
    }

    /// 取消订单。
    pub fn cancel(&mut self, value: u64) {
        if value == 0 {
            self.delegate.clear()
        } else {
            self.delegate.remove(&value);
        }
    }

    /// 更新。
    pub fn update(&mut self) {
        // 处理委托
        for (
            product,
            PriceData {
                time, high, low, ..
            },
        ) in self.product.iter().map(|v| (v.0.as_str(), v.1.to_owned()))
        {
            self.delegate.retain(|_, delegate| {
                if delegate.product != product {
                    return true;
                }

                if delegate.side == Side::BuyLong || delegate.side == Side::SellShort {
                    // 开仓委托
                    if delegate.side == Side::BuyLong && low <= delegate.price
                        || delegate.side == Side::SellShort && high >= delegate.price
                    {
                        // 查找现有仓位
                        let position = self
                            .position
                            .iter_mut()
                            .find(|v| v.0.product == delegate.product);

                        // 计算开仓均价
                        let (new_side, new_price, new_quantity, new_margin, append_margin) =
                            match position.as_ref() {
                                Some(v) => {
                                    let mut quantity = v.0.quantity * v.0.side.factor()
                                        + delegate.quantity * delegate.side.factor();

                                    let side = if quantity == 0.0 {
                                        quantity = delegate.quantity;
                                        delegate.side
                                    } else if quantity > 0.0 {
                                        Side::BuyLong
                                    } else {
                                        Side::SellShort
                                    };

                                    let open_price = ((v.0.open_price * v.0.quantity)
                                        + (delegate.price * delegate.quantity))
                                        / (v.0.quantity + delegate.quantity);

                                    let append_margin = (v.0.margin
                                        - v.0.quantity / self.config.lever as f64)
                                        + (delegate.margin
                                            - delegate.quantity / self.config.lever as f64);

                                    quantity = quantity.abs();

                                    (
                                        side,
                                        open_price,
                                        quantity,
                                        quantity / self.config.lever as f64 + append_margin,
                                        append_margin,
                                    )
                                }
                                None => (
                                    delegate.side,
                                    delegate.price,
                                    delegate.quantity,
                                    delegate.margin,
                                    delegate.margin - delegate.quantity / self.config.lever as f64,
                                ),
                            };

                        // 做多强平价格 = 入场价格 × (1 - 初始保证金率 + 维持保证金率) - (追加保证金 / 仓位数量) + 吃单手续费
                        // 做空强平价格 = 入场价格 × (1 + 初始保证金率 - 维持保证金率) + (追加保证金 / 仓位数量) - 吃单手续费
                        // 初始保证金率 = 1 / 杠杆
                        // 追加保证金 = 账户余额 - 初始化保证金
                        // 初始保证金 = 入场价格 / 杠杆
                        let imr = 1.0 / self.config.lever as f64;
                        let mmr = self.config.maintenance;
                        let liquidation_price = if new_side == Side::BuyLong {
                            new_price * (1.0 - imr + mmr)
                                - (append_margin / (new_quantity / new_price))
                                + delegate.quantity * self.config.close_fee
                        } else {
                            new_price * (1.0 + imr - mmr)
                                + (append_margin / (new_quantity / new_price))
                                - delegate.quantity * self.config.close_fee
                        };

                        // 仓位记录
                        let cp = SubPosition {
                            side: delegate.side,
                            price: delegate.price,
                            quantity: delegate.quantity,
                            margin: delegate.margin,
                            fee: delegate.quantity * self.config.open_fee,
                            profit: 0.0,
                            profit_ratio: 0.0,
                            time,
                        };

                        match position {
                            Some((position, sub_delegate)) => {
                                // 已经存在仓位
                                position.side = new_side;
                                position.open_price = new_price;
                                position.quantity = new_quantity;
                                position.margin = new_margin;
                                position.liquidation_price = liquidation_price;
                                position.list.push(cp);

                                // 订单附带的止盈委托
                                if delegate.stop_profit_condition != 0.0 {
                                    sub_delegate.push(SubDelegate {
                                        side: delegate.side.neg(),
                                        quantity: delegate.quantity,
                                        condition: delegate.stop_profit_condition,
                                        price: delegate.stop_profit,
                                    });
                                }

                                if delegate.stop_loss_condition != 0.0 {
                                    sub_delegate.push(SubDelegate {
                                        side: delegate.side.neg(),
                                        quantity: delegate.quantity,
                                        condition: delegate.stop_loss_condition
                                            * delegate.side.neg().factor(),
                                        price: delegate.stop_loss,
                                    });
                                }
                            }
                            None => {
                                // 新建仓位
                                let mut position = Position {
                                    product: delegate.product.clone(),
                                    lever: self.config.lever,
                                    side: new_side,
                                    open_price: new_price,
                                    quantity: new_quantity,
                                    margin: new_margin,
                                    liquidation_price,
                                    close_price: 0.0,
                                    profit: 0.0,
                                    profit_ratio: 0.0,
                                    fee: 0.0,
                                    open_time: time,
                                    close_time: 0,
                                    list: Vec::new(),
                                };

                                position.list.push(cp);

                                let mut sub_delegate = Vec::new();

                                // 订单附带的止盈止损委托
                                if delegate.stop_profit_condition != 0.0 {
                                    sub_delegate.push(SubDelegate {
                                        side: delegate.side.neg(),
                                        quantity: delegate.quantity,
                                        condition: delegate.stop_profit_condition
                                            * delegate.side.factor(),
                                        price: delegate.stop_profit,
                                    });
                                }

                                if delegate.stop_profit_condition != 0.0 {
                                    sub_delegate.push(SubDelegate {
                                        side: delegate.side.neg(),
                                        quantity: delegate.quantity,
                                        condition: delegate.stop_loss_condition
                                            * delegate.side.neg().factor(),
                                        price: delegate.stop_loss,
                                    });
                                }

                                self.position.push((position, sub_delegate));
                            }
                        }

                        return false;
                    }
                } else if let Some(v) = self
                    .position
                    .iter_mut()
                    .find(|v| v.0.product == delegate.product)
                    .map(|v| &mut v.1)
                {
                    // 平仓委托
                    v.push(SubDelegate {
                        side: delegate.side,
                        quantity: delegate.quantity,
                        condition: delegate.price,
                        price: 0.0,
                    });

                    return false;
                }

                true
            });
        }

        // 处理止盈止损
        for (
            product,
            PriceData {
                time, high, low, ..
            },
        ) in self.product.iter().map(|v| (v.0.as_str(), v.1.to_owned()))
        {
            for i in (0..self.position.len()).rev() {
                let (position, sub_delegate) = &mut self.position[i];
                if position.product == product {
                    for i in (0..sub_delegate.len()).rev() {
                        if position.quantity == 0.0 {
                            break;
                        }

                        let delegate = &sub_delegate[i];

                        if delegate.side == Side::BuySell
                            && (delegate.condition >= 0.0 && high >= delegate.condition
                                || delegate.condition <= 0.0 && low <= delegate.condition.abs())
                            || delegate.side == Side::SellLong
                                && (delegate.condition >= 0.0 && high >= delegate.condition
                                    || delegate.condition <= 0.0 && low <= delegate.condition.abs())
                        {
                            if delegate.price == 0.0 {
                                // 限价触发，市价委托
                                let margin =
                                    position.margin * delegate.quantity / position.quantity;

                                let fee = delegate.quantity * self.config.close_fee;

                                let profit = (delegate.condition.abs() - position.open_price)
                                    * delegate.quantity
                                    / position.open_price;

                                // 只修改会影响下单的属性，其他属性在完成平仓的时候计算。
                                position.quantity -= delegate.quantity;

                                position.margin -= margin;

                                self.balance += profit;

                                self.balance += margin;

                                self.balance -= fee;

                                position.list.push(SubPosition {
                                    side: delegate.side,
                                    price: delegate.condition.abs(),
                                    quantity: delegate.quantity,
                                    margin,
                                    fee,
                                    profit,
                                    profit_ratio: profit / margin,
                                    time,
                                });
                            } else {
                                // 限价触发，限价委托
                                sub_delegate.push(SubDelegate {
                                    side: delegate.side,
                                    quantity: delegate.quantity,
                                    condition: delegate.price,
                                    price: 0.0,
                                })
                            }

                            sub_delegate.swap_remove(i);
                        }
                    }

                    if position.quantity == 0.0 {
                        let mut position = self.position.swap_remove(i).0;

                        position.profit = position.list.iter().map(|v| v.profit).sum();

                        position.profit_ratio = position.list.iter().map(|v| v.profit_ratio).sum();

                        position.fee = position.list.iter().map(|v| v.fee).sum();

                        let mut max_quantity = 0.0;

                        let mut sum_quantity = 0.0;

                        let mut max_margin = 0.0;

                        let mut sum_margin = 0.0;

                        position
                            .list
                            .iter()
                            .filter(|v| v.side == Side::BuyLong || v.side == Side::SellShort)
                            .for_each(|v| {
                                sum_quantity += v.quantity * v.side.factor();

                                if sum_quantity.abs() > max_quantity {
                                    max_quantity = sum_quantity.abs();
                                }

                                sum_margin += v.margin * v.side.factor();

                                if sum_margin.abs() > max_margin {
                                    max_margin = sum_margin.abs();
                                }
                            });

                        // 最大持仓量
                        position.quantity = max_quantity;

                        // 最大保证金
                        position.margin = max_margin;

                        position.close_price = position.list.last().unwrap().price;

                        position.close_time = time;

                        self.history.push(position);
                    }
                }
            }
        }

        // 处理强平
        for (
            product,
            PriceData {
                time, high, low, ..
            },
        ) in self.product.iter().map(|v| (v.0.as_str(), v.1.to_owned()))
        {
            for i in (0..self.position.len()).rev() {
                let position = &self.position[i].0;
                if position.product == product
                    && (position.side == Side::BuyLong && low <= position.liquidation_price
                        || position.side == Side::SellShort && high >= position.liquidation_price)
                {
                    let mut position = self.position.swap_remove(i).0;

                    position.profit = -position.margin;

                    position.profit_ratio = position.profit / position.margin;

                    position.fee = position.list.iter().map(|v| v.fee).sum();

                    let mut max_quantity = 0.0;

                    let mut sum_quantity = 0.0;

                    let mut max_margin = 0.0;

                    let mut sum_margin = 0.0;

                    position
                        .list
                        .iter()
                        .filter(|v| v.side == Side::BuyLong || v.side == Side::SellShort)
                        .for_each(|v| {
                            sum_quantity += v.quantity * v.side.factor();

                            if sum_quantity.abs() > max_quantity {
                                max_quantity = sum_quantity.abs();
                            }

                            sum_margin += v.margin * v.side.factor();

                            if sum_margin.abs() > max_margin {
                                max_margin = sum_margin.abs();
                            }
                        });

                    // 最大持仓量
                    position.quantity = max_quantity;

                    // 最大保证金
                    position.margin = max_margin;

                    position.close_price = position.liquidation_price;

                    position.close_time = time;

                    position.list.push(SubPosition {
                        side: position.side.neg(),
                        price: position.liquidation_price,
                        quantity: position.quantity,
                        margin: position.margin,
                        fee: 0.0,
                        profit: position.profit,
                        profit_ratio: position.profit_ratio,
                        time: position.close_time,
                    });

                    self.history.push(position);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PriceData {
    /// 面值。
    unit: f64,

    /// 时间。
    time: u64,

    /// 最高价。
    high: f64,

    /// 最低价。
    low: f64,

    /// 收盘价。
    close: f64,
}

#[derive(Debug, Clone, Copy)]
struct SubDelegate {
    /// 方向。
    side: Side,

    /// 持仓量。
    quantity: f64,

    /// 条件，整数表示大于等于，负数表示小于等于。
    condition: f64,

    /// 价格。
    price: f64,
}

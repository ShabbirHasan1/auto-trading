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
        // TODO: 考虑删除 neg
        // TODO: position_mode 未经测试
        // TODO: 最大持仓量和 fee 计算冗余

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

        // 面值
        let unit = self.bourse.get_unit(product).await?;

        // 撮合引擎
        let me = std::cell::RefCell::new(MatchmakingEngine::new(self.config));

        me.borrow_mut().product(product, unit);

        for index in (0..time.len()).rev().into_iter() {
            let time = time[index];
            let open = Source::new(&open[index..]);
            let high = Source::new(&high[index..]);
            let low = Source::new(&low[index..]);
            let close = Source::new(&close[index..]);

            me.borrow_mut().ready(product, time, close[0]);

            let order = |side,
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
            };

            let cancel = |value: u64| me.borrow_mut().cancel(value);

            let mut cx = Context {
                product,
                level: strategy_level,
                unit,
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

            me.borrow_mut().update();
        }

        Ok(me.into_inner().history)
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
    /// 余额。
    balance: f64,

    /// 交易配置。
    config: Config,

    /// 产品，面值，时间，价格
    product: std::collections::HashMap<String, (f64, u64, f64)>,

    /// 仓位，开仓委托。
    delegate: std::collections::HashMap<u64, Delegate>,

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
            product: std::collections::HashMap::new(),
            delegate: std::collections::HashMap::new(),
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
        self.product.insert(product.to_string(), (unit, 0, 0.0));
    }

    /// 准备。
    /// 在调用委托之前，需要准备。
    /// 在准备之前，需要插入产品。
    ///
    /// * `time` 时间。
    /// * `value` 交易产品，价格。
    pub fn ready<S>(&mut self, product: S, time: u64, price: f64)
    where
        S: AsRef<str>,
    {
        let product = product.as_ref();
        let temp = self
            .product
            .get_mut(product)
            .expect(&format!("no product: {}", product));
        temp.1 = time;
        temp.2 = price;
    }

    /// 下单。
    /// 如果做多限价大于当前价格，那么价格大于等于限价的时候才会成交。
    /// 如果做空限价小于当前价格，那么价格小于等于限价的时候才会成交。
    /// 如果策略在价格到达 [`Config`] 止盈止损目标位之前没有平仓操作，则仓位会进行平仓操作。
    /// 开平仓模式和买卖模式都应该使用 [`Side::BuySell`] 和 [`Side::SellLong`] 进行平仓操作，这相当只减仓，而不会开新的仓位。
    /// 平仓不会导致仓位反向开单，平仓数量只能小于等于现有持仓数量。
    /// 如果在进行平仓操作后，现有的限价平仓委托的平仓量小于持仓量，则该委托将被撤销。
    /// 止盈止损委托为只减仓，平仓数量为 `quantity`，如果 [`Config`] 进行了平仓操作，那么止盈止损委托也会被撤销。
    ///
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `side` 订单方向。
    /// * `price` 委托价格，0 表示市价，其他表示限价。
    /// * `quantity` 委托数量，如果是开仓，则 0 表示使用 [`Config`] 的设置，[`Unit::Proportion`] 表示占用初始保证金的比例，如果是平仓，则 0 表示全部仓位，[`Unit::Proportion`] 表示占用仓位的比例。
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

        let (unit, _, close_price) = self
            .product
            .get(product)
            .ok_or(anyhow::anyhow!("no product: {}", product))?
            .to_owned();

        if side == Side::BuyLong || side == Side::SellShort {
            let price = if price == 0.0 { close_price } else { price };

            // 最小下单价值
            let min_unit = unit * price;

            // 策略仓位价值
            let mut strategy_quantity =
                quantity.to_quantity(self.config.initial_margin * self.config.lever as f64);

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
                    if strategy_quantity == 0.0 || strategy_quantity > min_unit {
                        strategy_quantity = min_unit;
                        min_unit
                    } else {
                        strategy_quantity
                    }
                } else {
                    match self.config.margin {
                        Unit::Quantity(v) => {
                            (v * self.config.quantity * self.config.lever as f64 / min_unit).floor()
                                * min_unit
                        }
                        Unit::Proportion(v) => {
                            (strategy_quantity / (v - self.config.quantity) / min_unit).floor()
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
                price + stop_profit_condition.to_quantity(price) * side.factor()
            };

            let stop_loss_condition = if stop_loss_condition == 0.0 {
                0.0
            } else {
                price + stop_loss_condition.to_quantity(price) * side.neg().factor()
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

            let id = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;

            self.delegate.insert(
                id,
                Delegate {
                    product: product.to_string(),
                    isolated: self.config.isolated,
                    lever: self.config.lever,
                    side,
                    price,
                    quantity,
                    margin,
                    stop_profit_condition,
                    stop_loss_condition,
                    stop_profit: price + stop_profit.to_quantity(price) * side.factor(),
                    stop_loss: price + stop_loss.to_quantity(price) * side.neg().factor(),
                },
            );

            return Ok(id);
        }

        if let Some(v) = self
            .position
            .iter()
            .find(|v| v.0.product == product)
            .map(|v| &v.0)
        {
            let price = if price == 0.0 { close_price } else { price };

            // 最小下单价值
            let min_unit = unit * v.close_price;

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

            let id = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;

            self.delegate.insert(
                id,
                Delegate {
                    product: product.to_string(),
                    isolated: self.config.isolated,
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

            return Ok(id);
        }

        anyhow::bail!("no position: {}", product);
    }

    /// 取消订单。
    pub fn cancel(&mut self, value: u64) {
        self.delegate.remove(&value);
    }

    /// 更新。
    pub fn update(&mut self) {
        // 处理强平
        'a: for (product, (_, time, price)) in
            self.product.iter().map(|v| (v.0.as_str(), v.1.to_owned()))
        {
            for i in (0..self.position.len()).rev() {
                let position = &self.position[i].0;
                if position.product == product
                    && (position.side == Side::BuyLong && price <= position.liquidation_price
                        || position.side == Side::SellShort && price >= position.liquidation_price)
                {
                    if self.config.isolated {
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
                                if sum_quantity > max_quantity {
                                    max_quantity = sum_quantity;
                                }
                                sum_margin += v.margin * v.side.factor();
                                if sum_margin > max_margin {
                                    max_margin = sum_margin;
                                }
                            });

                        // 最大持仓量
                        position.quantity = max_quantity.abs();

                        // 最大保证金
                        position.margin = max_margin.abs();

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
                    } else {
                        for (product, (_, time, _)) in
                            self.product.iter().map(|v| (v.0.as_str(), v.1.to_owned()))
                        {
                            for i in (0..self.position.len()).rev() {
                                let position = &self.position[i].0;
                                if position.product == product {
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
                                        .filter(|v| {
                                            v.side == Side::BuyLong || v.side == Side::SellShort
                                        })
                                        .for_each(|v| {
                                            sum_quantity += v.quantity * v.side.factor();
                                            if sum_quantity > max_quantity {
                                                max_quantity = sum_quantity;
                                            }
                                            sum_margin += v.margin * v.side.factor();
                                            if sum_margin > max_margin {
                                                max_margin = sum_margin;
                                            }
                                        });

                                    // 最大持仓量
                                    position.quantity = max_quantity.abs();

                                    // 最大保证金
                                    position.margin = max_margin.abs();

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
                            break 'a;
                        }
                    }
                }
            }
        }

        // 处理委托
        for (product, (_, time, price)) in
            self.product.iter().map(|v| (v.0.as_str(), v.1.to_owned()))
        {
            self.delegate.retain(|_, delegate| {
                if delegate.product != product {
                    return true;
                }

                if delegate.side == Side::BuyLong || delegate.side == Side::SellShort {
                    if delegate.side == Side::BuyLong && price <= delegate.price
                        || delegate.side == Side::SellShort && price >= delegate.price
                    {
                        if delegate.isolated {
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
                            let (new_side, new_price, new_quantity, new_margin, append_margin) =
                                match position.as_ref() {
                                    Some(v) => {
                                        let mut quantity = v.0.quantity * v.0.side.factor()
                                            + delegate.quantity * delegate.side.factor();

                                        let side = if quantity == 0.0 {
                                            delegate.side
                                        } else if quantity > 0.0 {
                                            Side::BuyLong
                                        } else {
                                            Side::SellShort
                                        };

                                        let open_price =
                                            ((v.0.open_price * v.0.quantity * side.factor())
                                                + (delegate.price
                                                    * delegate.quantity
                                                    * delegate.side.factor()))
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
                                        delegate.margin
                                            - delegate.quantity / self.config.lever as f64,
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
                                            is_config: false,
                                        });
                                    }

                                    if delegate.stop_loss_condition != 0.0 {
                                        sub_delegate.push(SubDelegate {
                                            side: delegate.side.neg(),
                                            quantity: delegate.quantity,
                                            condition: delegate.stop_loss_condition,
                                            price: delegate.stop_loss,
                                            is_config: false,
                                        });
                                    }

                                    // 确保仓位总是有一个的 Config 止盈止损委托
                                    if self.config.stop_profit != 0.0 {
                                        let delegate = sub_delegate
                                            .iter_mut()
                                            .find(|v| v.is_config && v.side == Side::BuySell)
                                            .unwrap();
                                        delegate.condition = new_price
                                            + self.config.stop_profit.to_quantity(new_price)
                                                * new_side.factor();
                                    }

                                    if self.config.stop_loss != 0.0 {
                                        let delegate = sub_delegate
                                            .iter_mut()
                                            .find(|v| v.is_config && v.side == Side::SellLong)
                                            .unwrap();
                                        delegate.condition = new_price
                                            + self.config.stop_loss.to_quantity(new_price)
                                                * new_side.factor();
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
                                            condition: delegate.stop_profit_condition,
                                            price: delegate.stop_profit,
                                            is_config: false,
                                        });
                                    }

                                    if delegate.stop_profit_condition != 0.0 {
                                        sub_delegate.push(SubDelegate {
                                            side: delegate.side.neg(),
                                            quantity: delegate.quantity,
                                            condition: delegate.stop_loss_condition,
                                            price: delegate.stop_loss,
                                            is_config: false,
                                        });
                                    }

                                    // 确保仓位总是有一个的 Config 止盈止损委托
                                    if self.config.stop_profit != 0.0 {
                                        sub_delegate.push(SubDelegate {
                                            side: new_side,
                                            quantity: new_quantity,
                                            condition: new_price
                                                + self.config.stop_profit.to_quantity(new_price)
                                                    * new_side.factor(),
                                            price: 0.0,
                                            is_config: true,
                                        });
                                    }

                                    if self.config.stop_loss != 0.0 {
                                        sub_delegate.push(SubDelegate {
                                            side: new_side,
                                            quantity: new_quantity,
                                            condition: new_price
                                                + self.config.stop_loss.to_quantity(new_price)
                                                    * new_side.factor(),
                                            price: 0.0,
                                            is_config: true,
                                        });
                                    }

                                    self.position.push((position, sub_delegate));
                                }
                            }

                            return false;
                        } else {
                            todo!("全仓")
                        }
                    }
                } else if let Some(v) = self
                    .position
                    .iter_mut()
                    .find(|v| v.0.product == delegate.product)
                    .map(|v| &mut v.1)
                {
                    v.push(SubDelegate {
                        side: delegate.side,
                        quantity: delegate.quantity,
                        condition: delegate.price,
                        price: 0.0,
                        is_config: false,
                    });

                    return false;
                }

                true
            });
        }

        // 处理止盈止损
        for (product, (_, time, price)) in
            self.product.iter().map(|v| (v.0.as_str(), v.1.to_owned()))
        {
            for i in (0..self.position.len()).rev() {
                let (position, sub_delegate) = &mut self.position[i];
                if position.product == product {
                    for i in (0..sub_delegate.len()).rev() {
                        let delegate = &sub_delegate[i];
                        if delegate.side == Side::BuySell && price >= delegate.condition
                            || delegate.side == Side::SellLong && price <= delegate.condition
                        {
                            if delegate.price == 0.0 {
                                // 限价触发，市价委托
                                let margin =
                                    position.margin * delegate.quantity / position.quantity;

                                let fee = delegate.quantity * self.config.close_fee;

                                let profit = (delegate.condition - position.open_price)
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
                                    price: delegate.condition,
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
                                    is_config: false,
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
                                if sum_quantity > max_quantity {
                                    max_quantity = sum_quantity;
                                }
                                sum_margin += v.margin * v.side.factor();
                                if sum_margin > max_margin {
                                    max_margin = sum_margin;
                                }
                            });

                        // 最大持仓量
                        position.quantity = max_quantity.abs();

                        // 最大保证金
                        position.margin = max_margin.abs();

                        position.close_price = price;

                        position.close_time = time;

                        self.history.push(position);
                    }
                }
            }
        }
    }
}

// TODO: 要不要 pub
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

    /// 是否 [`Config`] 的委托
    is_config: bool,
}

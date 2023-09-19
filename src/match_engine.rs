use crate::*;

/// 信息。
#[derive(Debug)]
struct Message {
    /// 最小下单数量。
    min_size: f64,

    /// 最小名义价值。
    min_notional: f64,

    /// K 线数据。
    k: K,

    /// 委托。
    delegate: Vec<DelegateTuple>,

    /// 仓位。
    position: Option<Position>,
}

#[derive(Debug)]
struct DelegateTuple {
    /// 委托 id。
    id: u64,

    /// 委托1。
    delegate1: Option<Delegate>,

    /// 委托2。
    delegate2: Option<Delegate>,

    /// 止盈委托。
    stop_profit_delegate: Option<Delegate>,

    /// 止损委托。
    stop_loss_delegate: Option<Delegate>,
}

/// 委托。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Delegate {
    /// 持仓方向。
    pub side: Side,

    /// 触发条件，正数表示大于等于，负数表示小于等于。
    pub condition: f64,

    /// 委托价格，不能为零。
    pub price: f64,

    /// 委托数量。
    pub quantity: f64,

    /// 保证金。
    pub margin: f64,
}

/// 撮合引擎。
/// 事件处理顺序：强平事件 -> 平仓委托 -> 开仓委托。
#[derive(Debug)]
pub struct MatchEngine {
    /// 余额。
    balance: f64,

    /// 委托 id。
    id: u64,

    /// 交易配置。
    config: Config,

    /// 产品，信息。
    product: Vec<(String, Message)>,

    /// 历史仓位。
    history: Vec<Position>,
}

impl MatchEngine {
    pub fn new(config: Config) -> Self {
        Self {
            balance: config.initial_margin,
            id: 0,
            config,
            product: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn history(&self) -> &Vec<Position> {
        &self.history
    }

    /// 插入产品。
    ///
    /// * `product` 交易产品。
    /// * `min_size` 最小下单数量。
    /// * `min_notional` 最小名义价值。
    pub fn product<S>(&mut self, product: S, min_size: f64, min_notional: f64)
    where
        S: AsRef<str>,
    {
        let product = product.as_ref();
        self.product.push((
            product.to_string(),
            Message {
                min_size,
                min_notional,
                k: K {
                    time: 114514,
                    open: 0.0,
                    high: 0.0,
                    low: 0.0,
                    close: 0.0,
                },
                delegate: Vec::new(),
                position: None,
            },
        ));
    }

    /// 准备。
    /// 在调用委托之前，需要准备。
    /// 在准备之前，需要插入产品。
    ///
    /// * `product` 交易产品。
    /// * `k` k 线数据。
    pub fn ready<S>(&mut self, product: S, k: K)
    where
        S: AsRef<str>,
    {
        let product = product.as_ref();
        self.product
            .iter_mut()
            .find(|v| v.0 == product)
            .map(|v| &mut v.1)
            .expect(&format!("no product: {}", product))
            .k = k;
    }

    /// 委托。
    /// 如果做多限价大于市价，那么价格大于等于限价的时候才会成交。
    /// 如果做空限价小于市价，那么价格小于等于限价的时候才会成交。
    /// 如果平多限价小于市价，那么价格小于等于限价的时候才会成交。
    /// 如果平空限价大于市价，那么价格大于等于限价的时候才会成交。
    /// 做多的止盈触发价不能小于等于委托价格。
    /// 做空的止盈触发价不能大于等于委托价格。
    /// 做多的止损触发价不能大于等于委托价格。
    /// 做空的止损触发价不能小于等于委托价格。
    /// 限价平仓委托不会在当前 k 线被成交。
    /// 平仓不会导致仓位反向开单，平仓数量只能小于等于现有持仓数量。
    /// 如果在进行平仓操作后，现有的限价平仓委托的平仓量小于持仓量，则该委托将被撤销。
    /// 平仓的止盈止损无效。
    ///
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `side` 委托方向。
    /// * `price` 委托价格，0 表示市价，其他表示限价。
    /// * `quantity` 委托数量，如果是开仓，则 0 表示使用 [`Config::quantity`] 的设置，如果是平仓，则 0 表示全部仓位，[`Unit::Proportion`] 表示占用仓位的比例。
    /// * `margin` 保证金，0 表示使用 [`Config::margin`] 的设置，保证金乘以杠杆必须大于仓位价值，即 [`Config::margin`] * [`Config::lever`] >= [`Config::quantity`]，超出仓位价值部分的保证金当作追加保证金。
    /// * `stop_profit_condition` 止盈触发价格，0 表示不设置，且 `stop_profit` 无效。
    /// * `stop_loss_condition` 止损触发价格，0 表示不设置，且 `stop_loss` 无效。
    /// * `stop_profit` 止盈委托价格，0 表示不设置，其他表示限价。
    /// * `stop_loss` 止损委托格，0 表示不设置，其他表示限价。
    /// * `return` 委托 id。
    pub fn order<S>(
        &mut self,
        product: S,
        side: Side,
        price: f64,
        quantity: Unit,
        margin: Unit,
        stop_profit_condition: Unit,
        stop_loss_condition: Unit,
        stop_profit: Unit,
        stop_loss: Unit,
    ) -> anyhow::Result<u64>
    where
        S: AsRef<str>,
    {
        let product = product.as_ref();

        let Message {
            min_size,
            min_notional,
            k,
            delegate,
            position,
        } = self
            .product
            .iter_mut()
            .find(|v| v.0 == product)
            .map(|v| &mut v.1)
            .ok_or(anyhow::anyhow!("no product: {}", product))?;

        if side == Side::BuyLong || side == Side::SellShort {
            let price = if price == 0.0 { k.close } else { price };

            // 1 张的价值
            let min_unit = price * *min_size;

            // 仓位价值
            let quantity = match if quantity == Unit::Zero {
                self.config.quantity
            } else {
                quantity
            } {
                Unit::Zero => min_unit,
                Unit::Contract(v) => min_unit * v as f64,
                Unit::Quantity(v) => v,
                Unit::Proportion(v) => self.config.initial_margin * v,
            };

            // 开仓数量不能小于最小下单数量。
            if quantity < min_unit {
                anyhow::bail!(
                    "product {}: open quantity < min size: {} < {}",
                    product,
                    quantity,
                    min_unit
                );
            }

            // 开仓价值不能小于最小名义价值
            if quantity < *min_notional {
                anyhow::bail!(
                    "product {}: open quantity < min notional: {} < {}",
                    product,
                    quantity,
                    min_notional
                );
            }

            // 投入的保证金
            let margin = match if margin == Unit::Zero {
                self.config.margin
            } else {
                margin
            } {
                Unit::Zero => min_unit / self.config.lever as f64,
                Unit::Contract(v) => min_unit * v as f64,
                Unit::Quantity(v) => v,
                Unit::Proportion(v) => self.config.initial_margin * v,
            };

            // 保证金必须足够维持仓位价值
            if (margin * self.config.lever as f64) < quantity {
                anyhow::bail!(
                    "product {}: margin * lever < open quantity: {} * {} < {}",
                    product,
                    margin,
                    self.config.lever,
                    quantity
                );
            }

            // 手续费
            let fee = quantity * self.config.open_fee;

            // 检查余额
            if self.balance < margin + fee {
                anyhow::bail!(
                    "product {}: insufficient fund: balance < margin + fee: {} < {} + {}",
                    product,
                    self.balance,
                    margin,
                    fee
                );
            }

            // 检查最大投入的保证金数量
            if self.config.max_margin != Unit::Zero {
                if let Some(position) = position {
                    let max_margin = match self.config.max_margin {
                        Unit::Zero => todo!("you are a big fool"),
                        Unit::Contract(v) => {
                            // 1 张相当的价值
                            let min_unit = position.open_price * *min_size;

                            // 1 张相当的保证金
                            let min_margin = min_unit / self.config.lever as f64;

                            // 1 张相当的保证金 * 张数
                            let max_margin = min_margin * v as f64;

                            max_margin
                        }
                        Unit::Quantity(v) => v,
                        Unit::Proportion(v) => self.config.initial_margin * v,
                    };

                    if position.margin + margin > max_margin {
                        anyhow::bail!(
                            "product {}: position margin + open margin > max margin: {} + {} > {:?}",
                            product,
                            position.margin,
                            margin,
                            max_margin,
                        );
                    }
                }
            }

            // 检查止盈止损参数
            if stop_profit_condition == Unit::Zero && stop_profit != Unit::Zero {
                anyhow::bail!(
                    "product {}: stop profit must be zero, because stop profit condition is zero",
                    product
                );
            }

            if stop_loss_condition == Unit::Zero && stop_loss != Unit::Zero {
                anyhow::bail!(
                    "product {}: stop loss must be zero, because stop loss condition is zero",
                    product
                )
            }

            let stop_profit_condition = match stop_profit_condition {
                Unit::Zero => Unit::Zero,
                Unit::Contract(v) => Unit::Quantity(min_unit * v as f64),
                Unit::Quantity(v) => Unit::Quantity(v),
                Unit::Proportion(v) => Unit::Quantity(if side == Side::BuyLong {
                    price + price * v
                } else {
                    price - price * v
                }),
            };

            let stop_loss_condition = match stop_loss_condition {
                Unit::Zero => Unit::Zero,
                Unit::Contract(v) => Unit::Quantity(min_unit * v as f64),
                Unit::Quantity(v) => Unit::Quantity(v),
                Unit::Proportion(v) => Unit::Quantity(if side == Side::BuyLong {
                    price - price * v
                } else {
                    price + price * v
                }),
            };

            let stop_profit = match stop_profit {
                Unit::Zero => stop_profit_condition,
                Unit::Contract(v) => Unit::Quantity(min_unit * v as f64),
                Unit::Quantity(v) => Unit::Quantity(v),
                Unit::Proportion(v) => Unit::Quantity(if side == Side::BuyLong {
                    price + price * v
                } else {
                    price - price * v
                }),
            };

            let stop_loss = match stop_loss {
                Unit::Zero => stop_loss_condition,
                Unit::Contract(v) => Unit::Quantity(min_unit * v as f64),
                Unit::Quantity(v) => Unit::Quantity(v),
                Unit::Proportion(v) => Unit::Quantity(if side == Side::BuyLong {
                    price - price * v
                } else {
                    price + price * v
                }),
            };

            if let Unit::Quantity(v) = stop_profit_condition {
                if v <= 0.0 {
                    anyhow::bail!("product {}: stop profit condition invalid: {}", product, v)
                }
            }

            if let Unit::Quantity(v) = stop_loss_condition {
                if v <= 0.0 {
                    anyhow::bail!("product {}: stop loss condition invalid: {}", product, v)
                }
            }

            if let Unit::Quantity(v) = stop_profit {
                if v <= 0.0 {
                    anyhow::bail!("product {}: stop profit invalid: {}", product, v)
                }
            }

            if let Unit::Quantity(v) = stop_loss {
                if v <= 0.0 {
                    anyhow::bail!("product {}: stop loss invalid: {}", product, v)
                }
            }

            // 检查止盈止损是否有利于仓位
            if side == Side::BuyLong {
                if let Unit::Quantity(v) = stop_profit_condition {
                    if v <= price {
                        anyhow::bail!(
                            "product {}: buy long, but stop profit condition <= open price: {} <= {}",
                            product,
                            v,
                            price
                        );
                    }
                }

                if let Unit::Quantity(v) = stop_loss_condition {
                    if v >= price {
                        anyhow::bail!(
                            "product {}: buy long, but stop loss condition >= open price: {} >= {}",
                            product,
                            v,
                            price
                        );
                    }
                }
            } else {
                if let Unit::Quantity(v) = stop_profit_condition {
                    if v >= price {
                        anyhow::bail!(
                            "product {}: sell short, but stop profit condition >= open price: {} >= {}",
                            product,
                            v,
                            price
                        );
                    }
                }

                if let Unit::Quantity(v) = stop_loss_condition {
                    if v <= price {
                        anyhow::bail!(
                            "product {}: sell short, but stop loss condition <= open price: {} <= {}",
                            product,
                            v,
                            price
                        );
                    }
                }
            };

            let stop_profit_condition = match stop_profit_condition {
                Unit::Zero => 0.0,
                Unit::Quantity(v) => v,
                _ => todo!("you are a big fool"),
            };

            let stop_loss_condition = match stop_loss_condition {
                Unit::Zero => 0.0,
                Unit::Quantity(v) => v,
                _ => todo!("you are a big fool"),
            };

            let stop_profit = match stop_profit {
                Unit::Zero => 0.0,
                Unit::Quantity(v) => v,
                _ => todo!("you are a big fool"),
            };

            let stop_loss = match stop_loss {
                Unit::Zero => 0.0,
                Unit::Quantity(v) => v,
                _ => todo!("you are a big fool"),
            };

            let condition = if price >= k.close { price } else { -price };

            let (delegate1, delegate2) = match position {
                Some(v) if v.side != side && v.quantity < quantity => (
                    Some(Delegate {
                        side: if v.side == Side::BuyLong {
                            Side::BuySell
                        } else {
                            Side::SellLong
                        },
                        condition: k.close,
                        price: k.close,
                        quantity: v.quantity,
                        margin: v.margin,
                    }),
                    Some(Delegate {
                        side,
                        condition,
                        price: condition,
                        quantity: quantity - v.quantity,
                        margin: margin - v.margin,
                    }),
                ),
                _ => (
                    Some(Delegate {
                        side,
                        condition,
                        price: condition,
                        quantity,
                        margin,
                    }),
                    None,
                ),
            };

            self.balance -= margin + fee;

            self.id += 1;

            delegate.push(DelegateTuple {
                id: self.id,
                delegate1,
                delegate2,
                stop_profit_delegate: (stop_profit_condition != 0.0).then_some(Delegate {
                    side: if side == Side::BuyLong {
                        Side::BuySell
                    } else {
                        Side::SellLong
                    },
                    condition: stop_profit_condition,
                    price: stop_profit,
                    quantity,
                    margin,
                }),
                stop_loss_delegate: (stop_loss_condition != 0.0).then_some(Delegate {
                    side: if side == Side::BuyLong {
                        Side::BuySell
                    } else {
                        Side::SellLong
                    },
                    condition: -stop_loss_condition,
                    price: stop_loss,
                    quantity,
                    margin,
                }),
            });

            return Ok(self.id);
        }

        if let Some(position) = position {
            let price = if price == 0.0 { k.close } else { price };

            // 最小下单价值
            let min_unit = position.open_price * *min_size;

            // 平仓量
            let quantity = match quantity {
                Unit::Zero => position.quantity,
                Unit::Contract(v) => min_unit * v as f64,
                Unit::Quantity(v) => v,
                Unit::Proportion(v) => (position.quantity * v / min_unit).floor() as f64,
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
            if quantity > position.quantity {
                anyhow::bail!(
                    "product {}: close quantity > open quantity: {} > {}",
                    product,
                    quantity,
                    position.quantity,
                );
            };

            self.id += 1;

            let condition = if price >= k.close { price } else { -price };

            delegate.push(DelegateTuple {
                id: self.id,
                delegate1: Some(Delegate {
                    side,
                    condition,
                    price: condition,
                    quantity,
                    margin: quantity / self.config.lever as f64,
                }),
                delegate2: None,
                stop_profit_delegate: None,
                stop_loss_delegate: None,
            });

            return Ok(self.id);
        }

        anyhow::bail!("no position: {}", product);
    }

    /// 取消委托。
    ///
    /// * `id` 委托 id。
    pub fn cancel(&mut self, id: u64) -> bool {
        if id == 0 {
            self.product.iter_mut().for_each(|v| v.1.delegate.clear());
            return true;
        }

        for i in self.product.iter_mut() {
            if let Some(v) = i.1.delegate.iter().position(|v| v.id == id) {
                i.1.delegate.remove(v);
                return true;
            }
        }

        false
    }

    /// 刷新。
    pub fn update(&mut self) {
        self.update_liquidation();

        self.update_close_delegate();

        self.update_open_delegate();
    }

    fn update_liquidation(&mut self) {
        for (
            ..,
            Message {
                k,
                delegate,
                position,
                ..
            },
        ) in self.product.iter_mut()
        {
            if position.is_none() {
                continue;
            }

            let current_position = position.as_mut().unwrap();

            if !(current_position.side == Side::BuyLong
                && k.low <= current_position.liquidation_price
                || current_position.side == Side::SellShort
                    && k.high >= current_position.liquidation_price)
            {
                continue;
            }

            // 删除所有平仓委托
            delegate.retain_mut(|v| {
                if let Some(delegate) = v.delegate1 {
                    if delegate.side == Side::BuyLong || delegate.side == Side::SellShort {
                        return true;
                    } else {
                        v.delegate1 = None;
                    }
                }

                if let Some(v) = v.delegate2 {
                    if v.side == Side::BuyLong || v.side == Side::SellShort {
                        return true;
                    }
                }

                false
            });

            let record = Record {
                side: if current_position.side == Side::BuyLong {
                    Side::BuySell
                } else {
                    Side::SellLong
                },
                price: current_position.liquidation_price,
                quantity: current_position.quantity,
                margin: current_position.margin,
                fee: 0.0,
                profit: -current_position.margin,
                profit_ratio: -1.0,
                time: k.time,
            };

            current_position.log.push(record);

            self.history
                .push(new_history_position(position.take().unwrap()));
        }
    }

    fn update_close_delegate(&mut self) {
        let mut handle = |k: &mut K,
                          delegate: &mut Option<Delegate>,
                          position: &mut Option<Position>| {
            if position.is_none() || delegate.is_none() {
                return;
            }

            let current_position = position.as_mut().unwrap();

            let current_delegate = delegate.as_mut().unwrap();

            loop {
                if !(current_delegate.side == Side::BuySell
                    && (current_delegate.condition >= 0.0 && k.high >= current_delegate.condition
                        || current_delegate.condition <= 0.0
                            && k.low <= current_delegate.condition.abs())
                    || current_delegate.side == Side::SellLong
                        && (current_delegate.condition >= 0.0
                            && k.high >= current_delegate.condition
                            || current_delegate.condition <= 0.0
                                && k.low <= current_delegate.condition.abs()))
                {
                    return;
                }

                if current_delegate.condition.abs() == current_delegate.price {
                    // 限价触发，市价委托
                    let profit = (current_delegate.condition.abs() - current_position.open_price)
                        * current_delegate.quantity
                        / current_position.open_price;

                    let record = Record {
                        side: current_delegate.side,
                        price: current_delegate.condition.abs(),
                        quantity: current_delegate.quantity,
                        margin: current_delegate.margin,
                        fee: current_delegate.quantity * self.config.close_fee,
                        profit,
                        profit_ratio: profit / current_delegate.margin,
                        time: k.time,
                    };

                    self.balance += record.profit + record.margin - record.fee;

                    current_position.quantity -= record.quantity;

                    current_position.margin -= record.margin;

                    current_position.log.push(record);

                    if current_position.quantity == 0.0 {
                        self.history
                            .push(new_history_position(position.take().unwrap()));
                    }

                    *delegate = None;

                    return;
                } else {
                    // 限价触发，限价委托
                    if current_delegate.side == Side::BuySell
                        && current_delegate.condition < current_delegate.price
                    {
                        //                   C
                        //          B        |
                        // A        |        |
                        // |        |        |
                        // open  condition  price
                        *current_delegate = Delegate {
                            side: current_delegate.side,
                            condition: current_delegate.price,
                            price: current_delegate.price,
                            quantity: current_delegate.quantity,
                            margin: current_delegate.margin,
                        };

                        continue;
                    }

                    if current_delegate.side == Side::BuySell
                        && current_delegate.condition > current_delegate.price
                    {
                        //
                        //          B
                        // A        |        C
                        // |        |        |
                        // open  condition  price
                        *current_delegate = Delegate {
                            side: current_delegate.side,
                            condition: -current_delegate.price,
                            price: -current_delegate.price,
                            quantity: current_delegate.quantity,
                            margin: current_delegate.margin,
                        };

                        continue;
                    }

                    if current_delegate.side == Side::SellLong
                        && current_delegate.condition > current_delegate.price
                    {
                        // A
                        // |        B
                        // |        |        C
                        // |        |        |
                        // open  condition  price
                        *current_delegate = Delegate {
                            side: current_delegate.side,
                            condition: -current_delegate.price,
                            price: -current_delegate.price,
                            quantity: current_delegate.quantity,
                            margin: current_delegate.margin,
                        };

                        continue;
                    }

                    if current_delegate.side == Side::SellLong
                        && current_delegate.condition < current_delegate.price
                    {
                        // A                 C
                        // |        B        |
                        // |        |        |
                        // |        |        |
                        // open  condition  price
                        *current_delegate = Delegate {
                            side: current_delegate.side,
                            condition: current_delegate.price,
                            price: current_delegate.price,
                            quantity: current_delegate.quantity,
                            margin: current_delegate.margin,
                        };

                        continue;
                    }
                }
            }
        };

        for (
            ..,
            Message {
                k,
                delegate,
                position,
                ..
            },
        ) in self.product.iter_mut()
        {
            for i in 0..delegate.len() {
                handle(k, &mut delegate[i].delegate1, position);

                handle(k, &mut delegate[i].stop_profit_delegate, position);

                handle(k, &mut delegate[i].stop_loss_delegate, position);
            }

            let mut i = 0;

            // 如果在进行平仓操作后，现有的限价平仓委托的平仓量小于持仓量，则该委托将被撤销
            while i < delegate.len() {
                if let Some(a) = delegate[i].delegate1 {
                    if a.side == Side::BuySell || a.side == Side::SellLong {
                        if let Some(b) = position {
                            if a.quantity > b.quantity {
                                delegate[i].delegate1 = None;
                            }
                        } else {
                            delegate[i].delegate1 = None;
                        }
                    }
                }

                if delegate[i].delegate1.is_none()
                    && delegate[i].delegate2.is_none()
                    && delegate[i].stop_profit_delegate.is_none()
                    && delegate[i].stop_loss_delegate.is_none()
                {
                    delegate.remove(i);
                } else {
                    i += 1;
                }
            }
        }
    }

    fn update_open_delegate(&mut self) {
        let handle = |product: &String,
                      k: &mut K,
                      delegate: &mut Option<Delegate>,
                      position: &mut Option<Position>| {
            let current_delegate = delegate.as_mut().unwrap();

            if !(current_delegate.side == Side::BuyLong
                && (current_delegate.condition >= 0.0 && k.high >= current_delegate.condition
                    || current_delegate.condition <= 0.0
                        && k.low <= current_delegate.condition.abs())
                || current_delegate.side == Side::SellShort
                    && (current_delegate.condition >= 0.0 && k.high >= current_delegate.condition
                        || current_delegate.condition <= 0.0
                            && k.low <= current_delegate.condition.abs()))
            {
                return;
            }

            // 计算开仓均价
            // 新方向，新价格，新持仓量，新保证金，追加保证金
            let (new_side, new_price, new_quantity, new_margin, append_margin) = match position {
                Some(v) if v.side == current_delegate.side => {
                    // 方向相同，表示加仓
                    let quantity = v.quantity + current_delegate.quantity;

                    // 开仓均价
                    let open_price = ((v.open_price * v.quantity)
                        + (current_delegate.condition * current_delegate.quantity))
                        / (v.quantity + current_delegate.quantity);

                    // 追加保证金
                    let append_margin = (v.margin - v.quantity / self.config.lever as f64)
                        + (current_delegate.margin
                            - current_delegate.quantity / self.config.lever as f64);

                    (
                        current_delegate.side,
                        open_price,
                        quantity,
                        quantity / self.config.lever as f64 + append_margin,
                        append_margin,
                    )
                }
                _ => (
                    current_delegate.side,
                    current_delegate.condition,
                    current_delegate.quantity,
                    current_delegate.margin,
                    current_delegate.margin - current_delegate.quantity / self.config.lever as f64,
                ),
            };

            // 计算吃单手续费是为了防止穿仓，即余额不够支付手续费的情况
            // 做多强平价格 = 入场价格 × (1 - 初始保证金率 + 维持保证金率) - (追加保证金 / 仓位数量) + 吃单手续费
            // 做空强平价格 = 入场价格 × (1 + 初始保证金率 - 维持保证金率) + (追加保证金 / 仓位数量) - 吃单手续费
            // 初始保证金率 = 1 / 杠杆
            // 追加保证金 = 账户余额 - 初始化保证金
            // 初始保证金 = 入场价格 / 杠杆
            let imr = 1.0 / self.config.lever as f64;
            let mmr = self.config.maintenance;
            let liquidation_price = if new_side == Side::BuyLong {
                new_price * (1.0 - imr + mmr) - (append_margin / (new_quantity / new_price))
                    + current_delegate.quantity * self.config.close_fee
            } else {
                new_price * (1.0 + imr - mmr) + (append_margin / (new_quantity / new_price))
                    - current_delegate.quantity * self.config.close_fee
            };

            // 交易记录
            let record = Record {
                side: current_delegate.side,
                price: current_delegate.price,
                quantity: current_delegate.quantity,
                margin: current_delegate.margin,
                fee: current_delegate.quantity * self.config.open_fee,
                profit: 0.0,
                profit_ratio: 0.0,
                time: k.time,
            };

            match position {
                Some(v) => {
                    // 如果已经存在仓位，则直接修改仓位
                    v.side = new_side;
                    v.open_price = new_price;
                    v.quantity = new_quantity;
                    v.margin = new_margin;
                    v.liquidation_price = liquidation_price;
                    v.log.push(record);
                }
                None => {
                    // 新建仓位
                    let mut current_position = Position {
                        product: product.clone(),
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
                        open_time: k.time,
                        close_time: 114514,
                        log: Vec::new(),
                    };

                    current_position.log.push(record);

                    position.replace(current_position);
                }
            };

            *delegate = None;
        };

        for (
            product,
            Message {
                k,
                delegate,
                position,
                ..
            },
        ) in self.product.iter_mut()
        {
            let mut i = 0;

            while i < delegate.len() {
                if let Some(v) = delegate[i].delegate1 {
                    if v.side == Side::BuyLong || v.side == Side::SellShort {
                        handle(product, k, &mut delegate[i].delegate1, position);
                    } else if delegate[i].delegate2.is_some() {
                        handle(product, k, &mut delegate[i].delegate2, position);
                    }
                }

                if delegate[i].delegate1.is_none()
                    && delegate[i].delegate2.is_none()
                    && delegate[i].stop_profit_delegate.is_none()
                    && delegate[i].stop_loss_delegate.is_none()
                {
                    delegate.remove(i);
                } else {
                    i += 1;
                }
            }
        }
    }
}

/// 根据 log 统计仓位。
///
/// * `最大持仓量`。
/// * `最大保证金`。
/// * `收益`。
/// * `收益率`。
/// * `手续费`。
/// * `最后平仓价格`。
/// * `最后平仓时间`。
fn new_history_position(mut position: Position) -> Position {
    position.profit = position.log.iter().map(|v| v.profit).sum();

    position.profit_ratio = position.log.iter().map(|v| v.profit_ratio).sum();

    position.fee = position.log.iter().map(|v| v.fee).sum();

    let mut max_quantity = 0.0;

    let mut sum_quantity = 0.0;

    let mut max_margin = 0.0;

    let mut sum_margin = 0.0;

    position.log.iter().for_each(|v| {
        sum_quantity += v.quantity
            * if v.side == Side::BuyLong || v.side == Side::SellShort {
                1.0
            } else {
                -1.0
            };

        if sum_quantity.abs() > max_quantity {
            max_quantity = sum_quantity.abs();
        }

        sum_margin += v.margin
            * if v.side == Side::BuyLong || v.side == Side::SellShort {
                1.0
            } else {
                -1.0
            };

        if sum_margin.abs() > max_margin {
            max_margin = sum_margin.abs();
        }
    });

    position.quantity = max_quantity;

    position.margin = max_margin;

    position.close_price = position.log.last().unwrap().price;

    position.close_time = position.log.last().unwrap().time;

    position
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_config1() {
        let config = Config::new()
            .initial_margin(1000.0)
            .quantity(Unit::Contract(2))
            .margin(Unit::Contract(6));

        let mut me = MatchEngine::new(config);

        me.product("BTC-USDT-SWAP", 0.01, 0.0);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 114514,
                open: 1000.0,
                high: 2500.0,
                low: 500.0,
                close: 2000.0,
            },
        );

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            0.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
        );

        println!("{:?}", result);

        println!("{:#?}", me);

        assert!(
            me.product[0].1.delegate[0].delegate1.unwrap()
                == Delegate {
                    side: Side::BuyLong,
                    condition: 2000.0,
                    price: 2000.0,
                    quantity: 40.0,
                    margin: 120.0,
                }
        );
    }

    #[test]
    fn test_config2() {
        let config = Config::new()
            .initial_margin(1000.0)
            .quantity(Unit::Quantity(80.0))
            .margin(Unit::Quantity(100.0));

        let mut me = MatchEngine::new(config);

        me.product("BTC-USDT-SWAP", 0.01, 0.0);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 114514,
                open: 1000.0,
                high: 2500.0,
                low: 500.0,
                close: 2000.0,
            },
        );

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            0.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
        );

        println!("{:?}", result);

        println!("{:#?}", me);

        assert!(
            me.product[0].1.delegate[0].delegate1.unwrap()
                == Delegate {
                    side: Side::BuyLong,
                    condition: 2000.0,
                    price: 2000.0,
                    quantity: 80.0,
                    margin: 100.0,
                }
        );
    }

    #[test]
    fn test_config3() {
        let config = Config::new()
            .initial_margin(1000.0)
            .quantity(Unit::Proportion(0.3))
            .margin(Unit::Proportion(0.6));

        let mut me = MatchEngine::new(config);

        me.product("BTC-USDT-SWAP", 0.01, 0.0);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 114514,
                open: 1000.0,
                high: 2500.0,
                low: 500.0,
                close: 2000.0,
            },
        );

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            0.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
        );

        println!("{:?}", result);

        println!("{:#?}", me);

        assert!(
            me.product[0].1.delegate[0].delegate1.unwrap()
                == Delegate {
                    side: Side::BuyLong,
                    condition: 2000.0,
                    price: 2000.0,
                    quantity: 300.0,
                    margin: 600.0,
                }
        );
    }

    #[test]
    fn test_config4() {
        let config = Config::new().initial_margin(1000.0);

        let mut me = MatchEngine::new(config);

        me.product("BTC-USDT-SWAP", 0.01, 0.0);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 114514,
                open: 1000.0,
                high: 2500.0,
                low: 500.0,
                close: 2000.0,
            },
        );

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            0.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
        );

        println!("{:?}", result);

        println!("{:#?}", me);

        assert!(
            me.product[0].1.delegate[0].delegate1.unwrap()
                == Delegate {
                    side: Side::BuyLong,
                    condition: 2000.0,
                    price: 2000.0,
                    quantity: 20.0,
                    margin: 20.0,
                }
        );
    }

    #[test]
    fn test_order1() {
        let config = Config::new().initial_margin(1000.0);

        let mut me = MatchEngine::new(config);

        me.product("BTC-USDT-SWAP", 0.01, 0.0);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 114514,
                open: 1000.0,
                high: 2500.0,
                low: 500.0,
                close: 2000.0,
            },
        );

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            0.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Quantity(2100.0),
            Unit::Quantity(1950.0),
            Unit::Quantity(3000.0),
            Unit::Quantity(1000.0),
        );

        println!("{:?}", result);

        println!("{:#?}", me);

        assert!(
            me.product[0].1.delegate[0].delegate1.unwrap()
                == Delegate {
                    side: Side::BuyLong,
                    condition: 2000.0,
                    price: 2000.0,
                    quantity: 20.0,
                    margin: 20.0,
                }
        );

        assert!(
            me.product[0].1.delegate[0].stop_profit_delegate.unwrap()
                == Delegate {
                    side: Side::BuySell,
                    condition: 2100.0,
                    price: 3000.0,
                    quantity: 20.0,
                    margin: 20.0,
                }
        );

        assert!(
            me.product[0].1.delegate[0].stop_loss_delegate.unwrap()
                == Delegate {
                    side: Side::BuySell,
                    condition: -1950.0,
                    price: 1000.0,
                    quantity: 20.0,
                    margin: 20.0,
                }
        );
    }

    #[test]
    fn test_order2() {
        let config = Config::new().initial_margin(1000.0);

        let mut me = MatchEngine::new(config);

        me.product("BTC-USDT-SWAP", 0.01, 0.0);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 114514,
                open: 1000.0,
                high: 2500.0,
                low: 500.0,
                close: 2000.0,
            },
        );

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            0.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Proportion(0.5),
            Unit::Proportion(0.3),
            Unit::Proportion(0.7),
            Unit::Proportion(0.5),
        );

        println!("{:?}", result);

        println!("{:#?}", me);

        assert!(
            me.product[0].1.delegate[0].delegate1.unwrap()
                == Delegate {
                    side: Side::BuyLong,
                    condition: 2000.0,
                    price: 2000.0,
                    quantity: 20.0,
                    margin: 20.0,
                }
        );

        assert!(
            me.product[0].1.delegate[0].stop_profit_delegate.unwrap()
                == Delegate {
                    side: Side::BuySell,
                    condition: 3000.0,
                    price: 3400.0,
                    quantity: 20.0,
                    margin: 20.0,
                }
        );

        assert!(
            me.product[0].1.delegate[0].stop_loss_delegate.unwrap()
                == Delegate {
                    side: Side::BuySell,
                    condition: -1400.0,
                    price: 1000.0,
                    quantity: 20.0,
                    margin: 20.0,
                }
        );
    }

    #[test]
    fn test_order_args() {
        let config = Config::new().initial_margin(1000.0);

        let mut me = MatchEngine::new(config);

        me.product("BTC-USDT-SWAP", 0.01, 0.0);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 114514,
                open: 1000.0,
                high: 2500.0,
                low: 500.0,
                close: 2000.0,
            },
        );

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            0.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Contract(1),
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
        );

        println!("{}", result.unwrap_err());

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            0.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Contract(1000),
            Unit::Zero,
            Unit::Zero,
        );

        println!("{}", result.unwrap_err());

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            0.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Contract(1),
            Unit::Zero,
        );

        println!("{}", result.unwrap_err());

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            0.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Contract(1),
        );

        println!("{}", result.unwrap_err());

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            0.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Quantity(1950.0),
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
        );

        println!("{}", result.unwrap_err());

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            0.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Quantity(2100.0),
            Unit::Zero,
            Unit::Zero,
        );

        println!("{}", result.unwrap_err());

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            2500.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Quantity(2100.0),
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
        );

        println!("{}", result.unwrap_err());

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            2500.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Quantity(3000.0),
            Unit::Zero,
            Unit::Zero,
        );

        println!("{}", result.unwrap_err());
    }

    #[test]
    fn test_update1() {
        let config = Config::new()
            .initial_margin(1000.0)
            .quantity(Unit::Contract(100))
            .margin(Unit::Quantity(200.0))
            .lever(100)
            .open_fee(0.0002)
            .close_fee(0.0005)
            .maintenance(0.004);

        let mut me = MatchEngine::new(config);

        me.product("BTC-USDT-SWAP", 0.01, 0.0);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 114514,
                open: 10000.0,
                high: 25000.0,
                low: 5000.0,
                close: 20000.0,
            },
        );

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            0.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Quantity(21000.0),
            Unit::Quantity(19500.0),
            Unit::Quantity(30000.0),
            Unit::Quantity(10000.0),
        );

        println!("{:?}", result);

        println!("{:#?}", me);

        me.update();

        println!("{:#?}", me);

        assert!(me.product[0].1.position.as_ref().unwrap().liquidation_price == 19890.0);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 1919810,
                open: 10000.0,
                high: 25000.0,
                low: 19800.0,
                close: 20000.0,
            },
        );

        me.update();

        println!("{:#?}", me);

        assert!(me.product[0].1.position.is_none());
    }

    #[test]
    fn test_update2() {
        let config = Config::new()
            .initial_margin(1000.0)
            .quantity(Unit::Contract(1))
            .lever(100)
            .open_fee(0.0002)
            .close_fee(0.0005)
            .maintenance(0.004);

        let mut me = MatchEngine::new(config);

        me.product("BTC-USDT-SWAP", 0.01, 0.0);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 114514,
                open: 10000.0,
                high: 25000.0,
                low: 5000.0,
                close: 20000.0,
            },
        );

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            20000.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
        );

        println!("{:?}", result);

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            21000.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
        );

        println!("{:?}", result);

        println!("{:#?}", me);

        me.update();

        println!("{:#?}", me);

        assert!(
            me.product[0]
                .1
                .position
                .as_ref()
                .unwrap()
                .open_price
                .floor()
                == 20512.0
        );
    }

    #[test]
    fn test_update3() {
        let config = Config::new()
            .initial_margin(1000.0)
            .quantity(Unit::Contract(1))
            .margin(Unit::Quantity(100.0))
            .lever(10)
            .open_fee(0.0002)
            .close_fee(0.0005)
            .maintenance(0.004);

        let mut me = MatchEngine::new(config);

        me.product("BTC-USDT-SWAP", 0.01, 0.0);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 1,
                open: 10000.0,
                high: 25000.0,
                low: 5000.0,
                close: 20000.0,
            },
        );

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            20000.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Quantity(20100.0),
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
        );

        println!("{:?}", result);

        println!("{:#?}", me);

        me.update();

        println!("{:#?}", me);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 1,
                open: 10000.0,
                high: 25000.0,
                low: 15000.0,
                close: 20000.0,
            },
        );

        me.update();

        println!("{:#?}", me);

        assert!(me.history[0].close_price == 20100.0);
    }

    #[test]
    fn test_update4() {
        let config = Config::new()
            .initial_margin(1000.0)
            .quantity(Unit::Contract(1))
            .margin(Unit::Quantity(100.0))
            .lever(10)
            .open_fee(0.0002)
            .close_fee(0.0005)
            .maintenance(0.004);

        let mut me = MatchEngine::new(config);

        me.product("BTC-USDT-SWAP", 0.01, 0.0);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 1,
                open: 10000.0,
                high: 25000.0,
                low: 5000.0,
                close: 20000.0,
            },
        );

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            20000.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Quantity(19900.0),
            Unit::Zero,
            Unit::Zero,
        );

        println!("{:?}", result);

        println!("{:#?}", me);

        me.update();

        println!("{:#?}", me);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 1,
                open: 10000.0,
                high: 25000.0,
                low: 15000.0,
                close: 20000.0,
            },
        );

        me.update();

        println!("{:#?}", me);

        assert!(me.history[0].close_price == 19900.0);
    }

    #[test]
    fn test_update5() {
        let config = Config::new()
            .initial_margin(1000.0)
            .quantity(Unit::Contract(1))
            .margin(Unit::Quantity(100.0))
            .lever(10)
            .open_fee(0.0002)
            .close_fee(0.0005)
            .maintenance(0.004);

        let mut me = MatchEngine::new(config);

        me.product("BTC-USDT-SWAP", 0.01, 0.0);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 1,
                open: 10000.0,
                high: 25000.0,
                low: 5000.0,
                close: 20000.0,
            },
        );

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            20000.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Quantity(20100.0),
            Unit::Zero,
            Unit::Quantity(20200.0),
            Unit::Zero,
        );

        println!("{:?}", result);

        println!("{:#?}", me);

        me.update();

        println!("{:#?}", me);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 2,
                open: 10000.0,
                high: 25000.0,
                low: 15000.0,
                close: 20000.0,
            },
        );

        me.update();

        println!("{:#?}", me);

        assert!(me.history[0].close_price == 20200.0);
    }

    #[test]
    fn test_update6() {
        let config = Config::new()
            .initial_margin(1000.0)
            .quantity(Unit::Contract(1))
            .margin(Unit::Quantity(100.0))
            .lever(100)
            .open_fee(0.0002)
            .close_fee(0.0005)
            .maintenance(0.004);

        let mut me = MatchEngine::new(config);

        me.product("BTC-USDT-SWAP", 0.01, 0.0);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 1,
                open: 10000.0,
                high: 25000.0,
                low: 5000.0,
                close: 20000.0,
            },
        );

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            25000.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
        );

        println!("{:?}", result);

        println!("{:#?}", me);

        me.update();

        println!("{:#?}", me);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 2,
                open: 10000.0,
                high: 25000.0,
                low: 15000.0,
                close: 20000.0,
            },
        );

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuySell,
            15200.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
        );

        me.update();

        println!("{:?}", result);

        println!("{:#?}", me);

        assert!(me.history[0].close_price != 15200.0);
    }

    #[test]
    fn test_update7() {
        let config = Config::new()
            .initial_margin(1000.0)
            .quantity(Unit::Contract(1))
            .margin(Unit::Quantity(100.0))
            .lever(100)
            .open_fee(0.0002)
            .close_fee(0.0005)
            .maintenance(0.004);

        let mut me = MatchEngine::new(config);

        me.product("BTC-USDT-SWAP", 0.01, 0.0);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 1,
                open: 10000.0,
                high: 25000.0,
                low: 5000.0,
                close: 20000.0,
            },
        );

        me.order(
            "BTC-USDT-SWAP",
            Side::BuyLong,
            25000.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
        )
        .unwrap();

        let result = me.order(
            "BTC-USDT-SWAP",
            Side::BuySell,
            50000.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
        );

        println!("{:?}", result.unwrap_err());

        me.update();

        println!("{:#?}", me);

        me.ready(
            "BTC-USDT-SWAP",
            K {
                time: 2,
                open: 30000.0,
                high: 45000.0,
                low: 20000.0,
                close: 40000.0,
            },
        );

        me.order(
            "BTC-USDT-SWAP",
            Side::BuySell,
            50000.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
        )
        .unwrap();

        println!("{:#?}", me);

        _ = me.order(
            "BTC-USDT-SWAP",
            Side::BuySell,
            44000.0,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
            Unit::Zero,
        );

        me.update();

        println!("{:#?}", me);

        assert!(me.history[0].close_price == 44000.0);

        assert!(me.product[0].1.delegate.is_empty());
    }
}

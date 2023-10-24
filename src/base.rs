use crate::*;
use std::ops;

/// k 线。
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct K {
    /// k 线的时间。
    pub time: u64,

    /// 开盘价。
    pub open: f64,

    /// 最高价。
    pub high: f64,

    /// 最低价。
    pub low: f64,

    /// 收盘价。
    pub close: f64,
}

impl std::fmt::Display for K {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\"time\": {}, \"open\": {}, \"high\": {}, \"low\": {}, \"close\": {}}}",
            time_to_string(self.time),
            self.open,
            self.high,
            self.low,
            self.close
        )
    }
}

/// 时间级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Level {
    /// 1 分。
    Minute1,

    /// 3 分。
    Minute3,

    /// 5 分。
    Minute5,

    /// 15 分。
    Minute15,

    /// 30 分。
    Minute30,

    /// 1 小时。
    Hour1,

    /// 2 小时。
    Hour2,

    /// 4 小时。
    Hour4,

    /// 6 小时。
    Hour6,

    /// 12 小时。
    Hour12,

    /// 1 天。
    Day1,

    /// 3 天。
    Day3,

    /// 1 周。
    Week1,

    /// 1 月。
    Month1,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Level::Minute1 => f.write_str("1 minute"),
            Level::Minute3 => f.write_str("3 minute"),
            Level::Minute5 => f.write_str("5 minute"),
            Level::Minute15 => f.write_str("15 minute"),
            Level::Minute30 => f.write_str("30 minute"),
            Level::Hour1 => f.write_str("1 hour"),
            Level::Hour2 => f.write_str("2 hour"),
            Level::Hour4 => f.write_str("4 hour"),
            Level::Hour6 => f.write_str("6 hour"),
            Level::Hour12 => f.write_str("12 hour"),
            Level::Day1 => f.write_str("1 day"),
            Level::Day3 => f.write_str("3 day"),
            Level::Week1 => f.write_str("1 meek"),
            Level::Month1 => f.write_str("1 month"),
        }
    }
}

/// 数据系列。
/// 索引越界将返回 f64::NAN。
/// 切片越界将返回 &[]。
#[derive(Debug)]
pub struct Source {
    pub inner: [f64],
}

impl Source {
    pub fn new(value: &[f64]) -> &Self {
        // 不要使用 AsRef
        unsafe { std::mem::transmute(value) }
    }

    fn index<T>(&self, index: T) -> &Source
    where
        T: std::slice::SliceIndex<[f64], Output = [f64]>,
    {
        unsafe {
            std::mem::transmute(
                std::mem::transmute::<_, &[f64]>(self)
                    .get(index)
                    .unwrap_or(&[]),
            )
        }
    }
}

impl std::ops::Deref for Source {
    type Target = [f64];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::fmt::Display for &Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self[0]))
    }
}

impl std::ops::Index<usize> for Source {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.get(index).unwrap_or(&f64::NAN)
    }
}

impl std::ops::Index<std::ops::Range<usize>> for Source {
    type Output = Source;

    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        self.index(index)
    }
}

impl std::ops::Index<std::ops::RangeFrom<usize>> for Source {
    type Output = Source;

    fn index(&self, index: std::ops::RangeFrom<usize>) -> &Self::Output {
        self.index(index)
    }
}

impl std::ops::Index<std::ops::RangeTo<usize>> for Source {
    type Output = Source;

    fn index(&self, index: std::ops::RangeTo<usize>) -> &Self::Output {
        self.index(index)
    }
}

impl std::ops::Index<std::ops::RangeFull> for Source {
    type Output = Source;

    fn index(&self, index: std::ops::RangeFull) -> &Self::Output {
        self.index(index)
    }
}

impl std::ops::Index<std::ops::RangeInclusive<usize>> for Source {
    type Output = Source;

    fn index(&self, index: std::ops::RangeInclusive<usize>) -> &Self::Output {
        self.index(index)
    }
}

impl std::ops::Index<std::ops::RangeToInclusive<usize>> for Source {
    type Output = Source;

    fn index(&self, index: std::ops::RangeToInclusive<usize>) -> &Self::Output {
        self.index(index)
    }
}

impl PartialEq<i64> for &Source {
    fn eq(&self, other: &i64) -> bool {
        &self.inner[0] == &(*other as f64)
    }
}

impl PartialEq<f64> for &Source {
    fn eq(&self, other: &f64) -> bool {
        &self.inner[0] == other
    }
}

impl PartialEq<[f64]> for Source {
    fn eq(&self, other: &[f64]) -> bool {
        &self.inner == other
    }
}

impl PartialEq for &Source {
    fn eq(&self, other: &Self) -> bool {
        self[0] == other[0]
    }
}

impl PartialOrd<i64> for &Source {
    fn partial_cmp(&self, other: &i64) -> Option<std::cmp::Ordering> {
        self[0].partial_cmp(&(*other as f64))
    }
}

impl PartialOrd<f64> for &Source {
    fn partial_cmp(&self, other: &f64) -> Option<std::cmp::Ordering> {
        self[0].partial_cmp(other)
    }
}

impl PartialOrd<[f64]> for Source {
    fn partial_cmp(&self, other: &[f64]) -> Option<std::cmp::Ordering> {
        self.inner.partial_cmp(other)
    }
}

impl PartialOrd for &Source {
    fn partial_cmp(&self, other: &&Source) -> Option<std::cmp::Ordering> {
        self[0].partial_cmp(&other[0])
    }
}

overload::overload!((a: &Source) + (b: i64) -> f64 { a[0] + b as f64 });

overload::overload!((a: &Source) - (b: i64) -> f64 { a[0] - b as f64 });

overload::overload!((a: &Source) * (b: i64) -> f64 { a[0] * b as f64 });

overload::overload!((a: &Source) / (b: i64) -> f64 { a[0] / b as f64 });

overload::overload!((a: &Source) % (b: i64) -> f64 { a[0] % b as f64 });

overload::overload!((a: &Source) + (b: f64) -> f64 { a[0] + b });

overload::overload!((a: &Source) - (b: f64) -> f64 { a[0] - b });

overload::overload!((a: &Source) * (b: f64) -> f64 { a[0] * b });

overload::overload!((a: &Source) / (b: f64) -> f64 { a[0] / b });

overload::overload!((a: &Source) % (b: f64) -> f64 { a[0] % b });

overload::overload!((a: i64) + (b: &Source) -> f64 { a as f64 + b[0] });

overload::overload!((a: i64) - (b: &Source) -> f64 { a as f64 - b[0] });

overload::overload!((a: i64) * (b: &Source) -> f64 { a as f64 * b[0] });

overload::overload!((a: i64) / (b: &Source) -> f64 { a as f64 / b[0] });

overload::overload!((a: i64) % (b: &Source) -> f64 { a as f64 % b[0] });

overload::overload!((a: f64) + (b: &Source) -> f64 { a + b[0] });

overload::overload!((a: f64) - (b: &Source) -> f64 { a - b[0] });

overload::overload!((a: f64) * (b: &Source) -> f64 { a * b[0] });

overload::overload!((a: f64) / (b: &Source) -> f64 { a / b[0] });

overload::overload!((a: f64) % (b: &Source) -> f64 { a % b[0] });

overload::overload!((a: &Source) + (b: &Source) -> f64 { a[0] + b[0] });

overload::overload!((a: &Source) - (b: &Source) -> f64 { a[0] - b[0] });

overload::overload!((a: &Source) * (b: &Source) -> f64 { a[0] * b[0] });

overload::overload!((a: &Source) / (b: &Source) -> f64 { a[0] / b[0] });

overload::overload!((a: &Source) % (b: &Source) -> f64 { a[0] % b[0] });

/// 时间范围。
#[derive(Debug, Clone, Copy)]
pub struct TimeRange {
    pub start: u64,
    pub end: u64,
}

impl From<u64> for TimeRange {
    fn from(value: u64) -> Self {
        Self {
            start: 0,
            end: value,
        }
    }
}

impl From<std::ops::Range<u64>> for TimeRange {
    fn from(value: std::ops::Range<u64>) -> Self {
        Self {
            start: value.start,
            end: value.end - 1,
        }
    }
}

impl From<std::ops::RangeFrom<u64>> for TimeRange {
    fn from(value: std::ops::RangeFrom<u64>) -> Self {
        Self {
            start: value.start,
            end: u64::MAX - 1,
        }
    }
}

impl From<std::ops::RangeTo<u64>> for TimeRange {
    fn from(value: std::ops::RangeTo<u64>) -> Self {
        Self {
            start: 0,
            end: value.end - 1,
        }
    }
}

impl From<std::ops::RangeFull> for TimeRange {
    fn from(_: std::ops::RangeFull) -> Self {
        Self {
            start: 0,
            end: u64::MAX - 1,
        }
    }
}

impl From<std::ops::RangeInclusive<u64>> for TimeRange {
    fn from(value: std::ops::RangeInclusive<u64>) -> Self {
        Self {
            start: *value.start(),
            end: *value.end(),
        }
    }
}

impl From<std::ops::RangeToInclusive<u64>> for TimeRange {
    fn from(value: std::ops::RangeToInclusive<u64>) -> Self {
        Self {
            start: 0,
            end: value.end,
        }
    }
}

/// 委托方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Side {
    /// 买入开多。
    BuyLong,

    /// 卖出开空。
    SellShort,

    /// 卖出平多。
    BuySell,

    /// 卖出平空。
    SellLong,
}

/// 交易记录。
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Record {
    /// 持仓方向。
    pub side: Side,

    /// 价格。
    pub price: f64,

    /// 持仓量。
    pub quantity: f64,

    /// 保证金。
    pub margin: f64,

    /// 手续费。
    pub fee: f64,

    /// 收益。
    pub profit: f64,

    /// 收益率。
    pub profit_ratio: f64,

    /// 时间。
    pub time: u64,
}

/// 仓位。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Position {
    /// 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    pub product: String,

    /// 杠杆。
    pub lever: u32,

    /// 持仓方向。
    pub side: Side,

    /// 开仓均价。
    pub open_price: f64,

    /// 持仓量，单位为币。
    pub quantity: f64,

    /// 保证金。
    pub margin: f64,

    /// 强平价格，0 表示不会强平。
    pub liquidation_price: f64,

    /// 平仓均价。
    pub close_price: f64,

    /// 收益。
    pub profit: f64,

    /// 收益率。
    pub profit_ratio: f64,

    /// 手续费。
    pub fee: f64,

    /// 开仓时间。
    pub open_time: u64,

    /// 平仓时间。
    pub close_time: u64,

    /// 交易记录。
    pub log: Vec<Record>,
}

/// 委托。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Delegate {
    /// 持仓方向。
    pub side: Side,

    /// 委托价格。
    pub price: Price,

    /// 委托数量。
    pub quantity: f64,

    /// 保证金。
    pub margin: f64,

    /// 追加保证金。
    pub append_margin: f64,
}

/// 委托状态。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DelegateState {
    /// 单个委托。
    Single(Delegate),

    /// 减仓委托，开仓委托。
    Hedging(Delegate, Delegate),

    /// 减仓委托，开仓委托，止盈委托。
    HedgingProfit(Delegate, Delegate, Delegate),

    /// 减仓委托，开仓委托，止损委托。
    HedgingLoss(Delegate, Delegate, Delegate),

    /// 减仓委托，开仓委托，止盈委托，止损委托。
    HedgingProfitLoss(Delegate, Delegate, Delegate, Delegate),

    /// 开仓委托，止盈委托。
    OpenProfit(Delegate, Delegate),

    /// 开仓委托，止损委托。
    OpenLoss(Delegate, Delegate),

    /// 开仓委托，止盈委托，止损委托。
    OpenProfitLoss(Delegate, Delegate, Delegate),

    /// 止盈委托，止损委托。
    ProfitLoss(Delegate, Delegate),
}

/// 价格。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Price {
    /// 大于等于触发价，市价。
    GreaterThanMarket(f64),

    /// 小于等于触发价，市价。
    LessThanMarket(f64),

    /// 大于等于触发价，限价。
    GreaterThanLimit(f64, f64),

    /// 小于等于触发价，限价。
    LessThanLimit(f64, f64),
}

/// 上下文环境。
pub struct Context<'a> {
    /// 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    pub product: &'a str,

    /// 最小委托数量。
    pub min_size: f64,

    /// 最小名义价值。
    pub min_notional: f64,

    /// 时间级别。
    pub level: Level,

    /// k 线的时间。
    pub time: u64,

    /// 开盘价数据系列。
    pub open: &'a Source,

    /// 最高价数据系列。
    pub high: &'a Source,

    /// 最低价数据系列。
    pub low: &'a Source,

    /// 收盘价数据系列。
    pub close: &'a Source,

    pub(crate) trading: &'a mut dyn Trading,
}

impl<'a> Context<'a> {
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
    /// * `side` 委托方向。
    /// * `price` 委托价格，0 表示市价，其他表示限价。
    /// * `return` 委托 id。
    pub fn order(&mut self, side: Side, price: f64) -> anyhow::Result<u64> {
        self.trading.order(
            self.product,
            side,
            price,
            Unit::Ignore,
            Unit::Ignore,
            Unit::Ignore,
            Unit::Ignore,
            Unit::Ignore,
            Unit::Ignore,
        )
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
    /// * `side` 委托方向。
    /// * `price` 委托价格，0 表示市价，其他表示限价。
    /// * `stop_profit_condition` 止盈触发价格，[`Unit::Ignore`] 表示不设置，且 `stop_profit` 无效。
    /// * `stop_loss_condition` 止损触发价格，[`Unit::Ignore`] 表示不设置，且 `stop_loss` 无效。
    /// * `return` 委托 id。
    pub fn order_profit_loss(
        &mut self,
        side: Side,
        price: f64,
        stop_profit_condition: Unit,
        stop_loss_condition: Unit,
    ) -> anyhow::Result<u64> {
        self.trading.order(
            self.product,
            side,
            price,
            Unit::Ignore,
            Unit::Ignore,
            stop_profit_condition,
            stop_loss_condition,
            Unit::Ignore,
            Unit::Ignore,
        )
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
    /// * `side` 委托方向。
    /// * `price` 委托价格，0 表示市价，其他表示限价。
    /// * `stop_profit_condition` 止盈触发价格，[`Unit::Ignore`] 表示不设置，且 `stop_profit` 无效。
    /// * `stop_loss_condition` 止损触发价格，[`Unit::Ignore`] 表示不设置，且 `stop_loss` 无效。
    /// * `stop_profit` 止盈委托价格，[`Unit::Ignore`] 表示不设置，其他表示限价。
    /// * `stop_loss` 止损委托格，[`Unit::Ignore`] 表示不设置，其他表示限价。
    /// * `return` 委托 id。
    pub fn order_profit_loss_condition(
        &mut self,
        side: Side,
        price: f64,
        stop_profit_condition: Unit,
        stop_loss_condition: Unit,
        stop_profit: Unit,
        stop_loss: Unit,
    ) -> anyhow::Result<u64> {
        self.trading.order(
            self.product,
            side,
            price,
            Unit::Ignore,
            Unit::Ignore,
            stop_profit_condition,
            stop_loss_condition,
            stop_profit,
            stop_loss,
        )
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
    /// * `side` 委托方向。
    /// * `price` 委托价格，0 表示市价，其他表示限价。
    /// * `quantity` 委托数量，单位为币，如果是开仓，则 [`Unit::Ignore`] 表示使用 [`Config::quantity`] 的设置，如果是平仓，则 [`Unit::Ignore`] 表示全部仓位，[`Unit::Proportion`] 表示占用仓位的比例。
    /// * `margin` 保证金，[`Unit::Ignore`] 表示使用 [`Config::margin`] 的设置，保证金乘以杠杆必须大于仓位价值，即 [`Config::margin`] * [`Config::lever`] >= [`Config::quantity`]，超出仓位价值部分的保证金当作追加保证金。
    /// * `return` 委托 id。
    pub fn order_quantity_margin(
        &mut self,
        side: Side,
        price: f64,
        quantity: Unit,
        margin: Unit,
    ) -> anyhow::Result<u64> {
        self.trading.order(
            self.product,
            side,
            price,
            quantity,
            margin,
            Unit::Ignore,
            Unit::Ignore,
            Unit::Ignore,
            Unit::Ignore,
        )
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
    /// * `side` 委托方向。
    /// * `price` 委托价格，0 表示市价，其他表示限价。
    /// * `quantity` 委托数量，单位为币，如果是开仓，则 [`Unit::Ignore`] 表示使用 [`Config::quantity`] 的设置，如果是平仓，则 [`Unit::Ignore`] 表示全部仓位，[`Unit::Proportion`] 表示占用仓位的比例。
    /// * `margin` 保证金，[`Unit::Ignore`] 表示使用 [`Config::margin`] 的设置，保证金乘以杠杆必须大于仓位价值，即 [`Config::margin`] * [`Config::lever`] >= [`Config::quantity`]，超出仓位价值部分的保证金当作追加保证金。
    /// * `stop_profit_condition` 止盈触发价格，[`Unit::Ignore`] 表示不设置，且 `stop_profit` 无效。
    /// * `stop_loss_condition` 止损触发价格，[`Unit::Ignore`] 表示不设置，且 `stop_loss` 无效。
    /// * `stop_profit` 止盈委托价格，[`Unit::Ignore`] 表示不设置，其他表示限价。
    /// * `stop_loss` 止损委托格，[`Unit::Ignore`] 表示不设置，其他表示限价。
    /// * `return` 委托 id。
    pub fn order_condition(
        &mut self,
        side: Side,
        price: f64,
        quantity: Unit,
        margin: Unit,
        stop_profit_condition: Unit,
        stop_loss_condition: Unit,
        stop_profit: Unit,
        stop_loss: Unit,
    ) -> anyhow::Result<u64> {
        self.trading.order(
            self.product,
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

    /// 撤销委托。
    /// 对于已成交的委托，将撤销止盈止损委托。
    ///
    /// * `id` 委托 id，0 表示取消所有委托。
    pub fn cancel(&mut self, id: u64) -> bool {
        self.trading.cancel(id)
    }

    /// 获取余额。
    pub fn balance(&self) -> f64 {
        self.trading.balance()
    }

    /// 获取委托。
    ///
    /// * `product` 委托 id。
    /// * `return` 委托的状态，如果委托不存在或者已经成交，则返回 None。
    pub fn delegate(&self, id: u64) -> Option<DelegateState> {
        self.trading.delegate(id)
    }

    /// 获取仓位。
    ///
    /// * `id` 委托 id。
    pub fn position(&self) -> Option<&Position> {
        self.trading.position(self.product)
    }
}

/// 交易接口。
pub trait Trading {
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
    /// * `quantity` 委托数量，单位为币，如果是开仓，则 [`Unit::Ignore`] 表示使用 [`Config::quantity`] 的设置，如果是平仓，则 [`Unit::Ignore`] 表示全部仓位，[`Unit::Proportion`] 表示占用仓位的比例。
    /// * `margin` 保证金，[`Unit::Ignore`] 表示使用 [`Config::margin`] 的设置，保证金乘以杠杆必须大于仓位价值，即 [`Config::margin`] * [`Config::lever`] >= [`Config::quantity`]，超出仓位价值部分的保证金当作追加保证金。
    /// * `stop_profit_condition` 止盈触发价格，[`Unit::Ignore`] 表示不设置，且 `stop_profit` 无效。
    /// * `stop_loss_condition` 止损触发价格，[`Unit::Ignore`] 表示不设置，且 `stop_loss` 无效。
    /// * `stop_profit` 止盈委托价格，[`Unit::Ignore`] 表示不设置，其他表示限价。
    /// * `stop_loss` 止损委托格，[`Unit::Ignore`] 表示不设置，其他表示限价。
    /// * `return` 委托 id。
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
    ) -> anyhow::Result<u64>;

    /// 撤销委托。
    /// 对于已成交的委托，将撤销止盈止损委托。
    ///
    /// * `id` 委托 id，0 表示取消所有委托。
    fn cancel(&mut self, id: u64) -> bool;

    /// 获取余额。
    fn balance(&self) -> f64;

    /// 获取委托。
    ///
    /// * `product` 委托 id。
    /// * `return` 委托的状态，如果委托不存在或者已经成交，则返回 None。
    fn delegate(&self, id: u64) -> Option<DelegateState>;

    /// 获取仓位。
    ///
    /// * `id` 委托 id。
    fn position(&self, product: &str) -> Option<&Position>;
}

/// 数量，比例
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    /// 忽略。
    Ignore,

    /// 数量。
    Quantity(f64),

    /// 比例。
    Proportion(f64),
}

/// 交易配置。
#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub initial_margin: f64,
    pub lever: u32,
    pub open_fee: f64,
    pub close_fee: f64,
    pub deviation: f64,
    pub maintenance: f64,
    pub quantity: Unit,
    pub margin: Unit,
    pub max_margin: Unit,
}

impl Config {
    pub fn new() -> Self {
        Config {
            initial_margin: 0.0,
            lever: 1,
            open_fee: 0.0,
            close_fee: 0.0,
            deviation: 0.0,
            maintenance: 0.0,
            quantity: Unit::Ignore,
            margin: Unit::Ignore,
            max_margin: Unit::Ignore,
        }
    }

    /// 初始保证金。
    pub fn initial_margin(mut self, value: f64) -> Self {
        self.initial_margin = value;
        self
    }

    /// 杠杆。
    pub fn lever(mut self, value: u32) -> Self {
        self.lever = value;
        self
    }

    /// 挂单的手续费率。
    pub fn open_fee(mut self, value: f64) -> Self {
        self.open_fee = value;
        self
    }

    /// 吃单的手续费率。
    pub fn close_fee(mut self, value: f64) -> Self {
        self.close_fee = value;
        self
    }

    /// 滑点率。
    pub fn deviation(mut self, value: f64) -> Self {
        self.deviation = value;
        self
    }

    /// 维持保证金率。
    pub fn maintenance(mut self, value: f64) -> Self {
        self.maintenance = value;
        self
    }

    /// 每次开仓的仓位价值。
    /// 默认为最小委托数量。
    ///
    /// * [`Unit::Quantity`] 数量，单位为币。
    /// * [`Unit::Proportion`] 占用初始化保证金的比例。
    pub fn quantity(mut self, value: Unit) -> Self {
        self.quantity = value.into();
        self
    }

    /// 每次开仓投入的保证金。
    /// 默认为开仓所需的最低成本。
    /// 保证金乘以杠杆必须大于仓位价值，即 [`Config::margin`] * [`Config::lever`] >= [`Config::quantity`]。
    /// 超出仓位价值部分的保证金当作追加保证金。
    ///
    /// * [`Unit::Quantity`] 数量，单位为法币。
    /// * [`Unit::Proportion`] 占用初始化保证金的比例。
    pub fn margin(mut self, value: Unit) -> Self {
        self.margin = value.into();
        self
    }

    /// 最大投入的保证金数量，超过后将开单失败。
    /// 默认为无限制。
    ///
    /// * [`Unit::Quantity`] 数量，例如 USDT。
    /// * [`Unit::Proportion`] 占用初始化保证金的比例。
    pub fn max_margin(mut self, value: Unit) -> Self {
        self.max_margin = value.into();
        self
    }
}

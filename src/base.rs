use crate::*;

/// K 线。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct K {
    /// K 线的时间。
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
    inner: [f64],
}

impl Source {
    pub fn new<'a, T>(value: T) -> &'a Self
    where
        T: AsRef<[f64]>,
    {
        <&Self>::from(value)
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

impl<T> From<T> for &Source
where
    T: AsRef<[f64]>,
{
    fn from(value: T) -> Self {
        unsafe { std::mem::transmute(value.as_ref()) }
    }
}

impl std::ops::Deref for Source {
    type Target = [f64];

    fn deref(&self) -> &Self::Target {
        &self.inner
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

/// 变量值。
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Number(f64),
    String(String),
    Array(Vec<Value>),
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Value::Number(value as f64)
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Value::Number(value as f64)
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Value::Number(value as f64)
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Value::Number(value as f64)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Number(value as f64)
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value::Number(value as f64)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Number(value as f64)
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Value::Number(value as f64)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Number(value as f64)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Value::Array(value)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Number(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
            Value::Array(v) => {
                write!(
                    f,
                    "[{}]",
                    v.iter()
                        .map(|v| format!("{}", v))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}

/// 时间范围。
pub struct TimeRange {
    pub start: u64,
    pub end: u64,
}

impl From<u64> for TimeRange {
    fn from(value: u64) -> Self {
        Self {
            start: value,
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

/// 订单方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Side {
    /// 获取用以计算多空仓位的乘数。
    pub fn factor(&self) -> f64 {
        match self {
            Side::BuyLong => 1.0,
            Side::SellShort => -1.0,
            Side::BuySell => panic!("buy sell cannot get factor"),
            Side::SellLong => panic!("sell long cannot get factor"),
        }
    }

    /// 获取仓位的反方向。
    pub fn neg(&self) -> Side {
        match self {
            Side::BuyLong => Side::BuySell,
            Side::SellShort => Side::SellLong,
            Side::BuySell => panic!("buy sell cannot get neg"),
            Side::SellLong => panic!("sell long cannot get neg"),
        }
    }
}

/// 委托。
#[derive(Debug, Clone)]
pub struct Delegate {
    /// 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    pub product: String,

    /// 逐仓。
    pub isolated: bool,

    /// 杠杆。
    pub lever: u32,

    /// 持仓方向。
    pub side: Side,

    /// 委托价格。
    pub price: f64,

    /// 委托数量。
    pub quantity: f64,

    /// 保证金。
    pub margin: f64,

    /// 止盈触发价。
    pub stop_profit_condition: f64,

    /// 止损触发价。
    pub stop_loss_condition: f64,

    /// 止盈委托价。
    pub stop_profit: f64,

    /// 止损委托价。
    pub stop_loss: f64,
}

/// 清单仓位。
#[derive(Debug, Clone)]
pub struct SubPosition {
    /// 持仓方向。
    pub side: Side,

    /// 均价。
    pub price: f64,

    /// 持仓量。
    pub quantity: f64,

    /// 保证金。
    pub margin: f64,

    /// 收益。
    pub profit: f64,

    /// 收益率。
    pub profit_ratio: f64,

    /// 时间。
    pub time: u64,
}

/// 仓位。
#[derive(Debug, Clone)]
pub struct Position {
    /// 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    pub product: String,

    /// 逐仓。
    pub isolated: bool,

    /// 杠杆。
    pub lever: u32,

    /// 持仓方向。
    pub side: Side,

    /// 开仓均价。
    pub open_price: f64,

    /// 持仓量。
    pub open_quantity: f64,

    /// 保证金。
    pub margin: f64,

    /// 强平价格。
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

    /// 交易清单。
    pub list: Vec<SubPosition>,
}

/// 上下文环境。
pub struct Context<'a> {
    /// 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    pub product: &'a str,

    /// 时间级别。
    pub level: Level,

    /// K 线的时间。
    pub time: u64,

    /// 开盘价数据系列。
    pub open: &'a Source,

    /// 最高价数据系列。
    pub high: &'a Source,

    /// 最低价数据系列。
    pub low: &'a Source,

    /// 收盘价数据系列。
    pub close: &'a Source,

    pub(crate) variable: &'a mut std::collections::HashMap<&'static str, Value>,

    pub(crate) order:
        &'a mut dyn FnMut(Side, f64, Unit, Unit, Unit, Unit, Unit) -> anyhow::Result<usize>,

    pub(crate) cancel: &'a dyn Fn(usize),

    pub(crate) new_context: &'a dyn Fn(&str, Level) -> &Context,
}

impl<'a> Context<'a> {
    /// 下单。
    /// 如果做多限价大于当前价格，那么价格大于等于限价的时候才会成交。
    /// 如果做空限价小于当前价格，那么价格小于等于限价的时候才会成交。
    /// 如果策略在价格到达 [`Config`] 止盈止损目标位之前没有平仓操作，则仓位会进行平仓操作。
    /// 开平仓模式和买卖模式都应该使用 [`Side::BuySell`] 和 [`Side::SellLong`] 进行平仓操作，这相当只减仓，而不会开新的仓位。
    /// 平仓不会导致仓位反向开单，平仓数量只能小于等于现有持仓数量。
    /// 如果在进行平仓操作后，现有的限价平仓委托的平仓量小于持仓量，则该委托将被撤销。
    ///
    /// * `side` 订单方向。
    /// * `price` 委托价格，0 表示市价，其他表示限价。
    /// * `return` 订单 id。
    pub fn order(&mut self, side: Side, price: f64) -> anyhow::Result<usize> {
        (self.order)(
            side,
            price,
            0.into(),
            0.into(),
            0.into(),
            0.into(),
            0.into(),
        )
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
    /// * `side` 订单方向。
    /// * `price` 委托价格，0 表示市价，其他表示限价。
    /// * `quantity` 委托数量，0 表示使用 [`Config`] 的设置，[`Unit::Proportion`] 表示占用初始保证金的比例。
    /// * `stop_profit_condition` 止盈触发价格，0 表示不设置，且 `stop_profit` 无效。
    /// * `stop_loss_condition` 止损触发价格，0 表示不设置，且 `stop_loss` 无效。
    /// * `stop_profit` 止盈委托价格，0 表示市价，其他表示限价。
    /// * `stop_loss` 止损委托格，0 表示市价，其他表示限价。
    /// * `return` 订单 id。
    pub fn order_condition<A, B, C, D, E>(
        &mut self,
        side: Side,
        price: f64,
        quantity: A,
        stop_profit_condition: B,
        stop_loss_condition: C,
        stop_profit: D,
        stop_loss: E,
    ) -> anyhow::Result<usize>
    where
        A: Into<Unit>,
        B: Into<Unit>,
        C: Into<Unit>,
        D: Into<Unit>,
        E: Into<Unit>,
    {
        (self.order)(
            side,
            price,
            quantity.into(),
            stop_profit_condition.into(),
            stop_loss_condition.into(),
            stop_profit.into(),
            stop_loss.into(),
        )
    }

    /// 撤销订单。
    /// 对于已完成的订单，将撤销止盈止损委托。
    ///
    /// * `id` 订单 id。
    pub fn cancel(&self, id: usize) {
        (self.cancel)(id)
    }

    /// 创建新的上下文环境，继承当前的上下文变量表。
    /// 如果要下单其他交易产品，则要将 [`crate::backtester::Backtester`] 中的 `other_product` 设置为 true，否则下单失败。
    ///
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `level` 时间级别。
    /// * `return` 上下文环境。
    pub fn new_context(&'a self, product: &'a str, level: Level) -> &Context {
        (self.new_context)(product, level)
    }
}

impl<'a> std::ops::Index<&'static str> for Context<'a> {
    type Output = Value;

    fn index(&self, index: &'static str) -> &Self::Output {
        debug_assert!(self.variable.contains_key(index), "变量不存在: {}", index);
        self.variable.get(index).unwrap()
    }
}

impl<'a> std::ops::IndexMut<&'static str> for Context<'a> {
    fn index_mut(&mut self, index: &'static str) -> &mut Self::Output {
        debug_assert!(self.variable.contains_key(index), "变量不存在: {}", index);
        self.variable.get_mut(index).unwrap()
    }
}

/// 数量或比例。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Quantity(f64),
    Proportion(f64),
}

impl Unit {
    /// 转换到数量。
    pub fn to_quantity(self, value: f64) -> f64 {
        match self {
            Self::Quantity(v) => v,
            Self::Proportion(v) => v * value,
        }
    }
}

impl From<i8> for Unit {
    fn from(value: i8) -> Self {
        Self::Quantity(value as f64)
    }
}

impl From<u8> for Unit {
    fn from(value: u8) -> Self {
        Self::Quantity(value as f64)
    }
}

impl From<i16> for Unit {
    fn from(value: i16) -> Self {
        Self::Quantity(value as f64)
    }
}

impl From<u16> for Unit {
    fn from(value: u16) -> Self {
        Self::Quantity(value as f64)
    }
}

impl From<i32> for Unit {
    fn from(value: i32) -> Self {
        Self::Quantity(value as f64)
    }
}

impl From<u32> for Unit {
    fn from(value: u32) -> Self {
        Self::Quantity(value as f64)
    }
}

impl From<i64> for Unit {
    fn from(value: i64) -> Self {
        Self::Quantity(value as f64)
    }
}

impl From<u64> for Unit {
    fn from(value: u64) -> Self {
        Self::Quantity(value as f64)
    }
}

impl From<f32> for Unit {
    fn from(value: f32) -> Self {
        Self::Quantity(value as f64)
    }
}

impl From<f64> for Unit {
    fn from(value: f64) -> Self {
        Self::Quantity(value)
    }
}

impl std::cmp::PartialEq<f64> for Unit {
    fn eq(&self, other: &f64) -> bool {
        match (self, other) {
            (Self::Quantity(l0), r0) => l0 == r0,
            (Self::Proportion(l0), r0) => l0 == r0,
        }
    }
}

/// 交易配置，参数可以不设置，这取决于你的策略。
/// 如果策略需要下单，但没有设置 `initial_margin` 和 `margin` 属性，则下单失败。
#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub initial_margin: f64,
    pub isolated: bool,
    pub position_mode: bool,
    pub lever: u32,
    pub fee: f64,
    pub deviation: f64,
    pub maintenance: f64,
    pub margin: Unit,
    pub max_margin: Unit,
    pub stop_profit: Unit,
    pub stop_loss: Unit,
}

impl Config {
    pub fn new() -> Self {
        Config {
            initial_margin: 0.0,
            isolated: false,
            position_mode: false,
            lever: 1,
            fee: 0.0,
            deviation: 1.0,
            maintenance: 1.0,
            margin: 0.into(),
            max_margin: 0.into(),
            stop_profit: 0.into(),
            stop_loss: 0.into(),
        }
    }

    /// 初始保证金。
    pub fn initial_margin(mut self, value: f64) -> Self {
        self.initial_margin = value;
        self
    }

    /// 逐仓。
    pub fn isolated(mut self, value: bool) -> Self {
        self.isolated = value;
        self
    }

    /// 仓位模式，true 表示开平仓模式，一个合约可同时持有多空两个方向的仓位，false 表示买卖模式，一个合约仅可持有一个方向的仓位。
    pub fn position_mode(mut self, value: bool) -> Self {
        self.position_mode = value;
        self
    }

    /// 杠杆。
    pub fn lever(mut self, value: u32) -> Self {
        self.lever = value;
        self
    }

    /// 进场和出场的手续费。
    pub fn fee(mut self, value: f64) -> Self {
        self.fee = value;
        self
    }

    /// 滑点比例。
    pub fn deviation(mut self, value: f64) -> Self {
        self.deviation = value;
        self
    }

    /// 维持保证金率。
    pub fn maintenance(mut self, value: f64) -> Self {
        self.maintenance = value;
        self
    }

    /// 每次开单投入的保证金。
    pub fn margin<T>(mut self, value: T) -> Self
    where
        T: Into<Unit>,
    {
        self.margin = value.into();
        self
    }

    /// 最大投入的保证金数量，超过后将开单失败。
    pub fn max_margin<T>(mut self, value: T) -> Self
    where
        T: Into<Unit>,
    {
        self.max_margin = value.into();
        self
    }

    /// 单笔止盈数量。
    pub fn stop_profit<T>(mut self, value: T) -> Self
    where
        T: Into<Unit>,
    {
        self.stop_profit = value.into();
        self
    }

    /// 单笔止损数量。
    pub fn stop_loss<T>(mut self, value: T) -> Self
    where
        T: Into<Unit>,
    {
        self.stop_loss = value.into();
        self
    }
}

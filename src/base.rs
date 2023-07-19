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

/// 时间级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Level {
    /// 1 分。
    Minute1,

    /// 5 分。
    Minute5,

    /// 15 分。
    Minute15,

    /// 30 分。
    Minute30,

    /// 1 小时。
    Hour1,

    /// 4 小时。
    Hour4,

    /// 1 天。
    Day1,

    /// 1 周。
    Week1,

    /// 1 月。
    Month1,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Level::Minute1 => f.write_str("1 minute"),
            Level::Minute5 => f.write_str("5 minute"),
            Level::Minute15 => f.write_str("15 minute"),
            Level::Minute30 => f.write_str("30 minute"),
            Level::Hour1 => f.write_str("1 hour"),
            Level::Hour4 => f.write_str("4 hour"),
            Level::Day1 => f.write_str("1 day"),
            Level::Week1 => f.write_str("1 meek"),
            Level::Month1 => f.write_str("1 month"),
        }
    }
}

/// 数据系列。
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
        unsafe { core::mem::transmute(value.as_ref()) }
    }
}

impl core::ops::Deref for Source {
    type Target = [f64];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl core::ops::Index<usize> for Source {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        self.inner.get(index).unwrap_or(&f64::NAN)
    }
}

impl core::ops::Index<core::ops::Range<usize>> for Source {
    type Output = Source;

    fn index(&self, index: core::ops::Range<usize>) -> &Self::Output {
        self.index(index)
    }
}

impl core::ops::Index<core::ops::RangeFrom<usize>> for Source {
    type Output = Source;

    fn index(&self, index: core::ops::RangeFrom<usize>) -> &Self::Output {
        self.index(index)
    }
}

impl core::ops::Index<core::ops::RangeTo<usize>> for Source {
    type Output = Source;

    fn index(&self, index: core::ops::RangeTo<usize>) -> &Self::Output {
        self.index(index)
    }
}

impl core::ops::Index<core::ops::RangeFull> for Source {
    type Output = Source;

    fn index(&self, index: core::ops::RangeFull) -> &Self::Output {
        self.index(index)
    }
}

impl core::ops::Index<core::ops::RangeInclusive<usize>> for Source {
    type Output = Source;

    fn index(&self, index: core::ops::RangeInclusive<usize>) -> &Self::Output {
        self.index(index)
    }
}

impl core::ops::Index<core::ops::RangeToInclusive<usize>> for Source {
    type Output = Source;

    fn index(&self, index: core::ops::RangeToInclusive<usize>) -> &Self::Output {
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
pub enum Side {
    /// 买入开多。
    BuyLong,

    /// 卖出开空。
    SellShort,

    /// 卖出平空。
    SellLong,

    /// 卖出平多。
    BuySell,
}

/// 仓位。
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

    /// 平仓均价。
    pub close_price: f64,

    /// 持仓量。
    pub open_quantity: f64,

    /// 平仓量。
    pub close_quantity: f64,

    /// 收益。
    pub profit: f64,

    /// 收益率。
    pub profit_ratio: f64,

    /// 开仓时间。
    pub open_time: u64,

    /// 平仓时间。
    pub close_time: u64,
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

    pub(crate) variable: &'a mut std::collections::BTreeMap<&'static str, Value>,

    pub(crate) order: &'a dyn Fn(Side, Unit, Unit, Unit, Unit) -> Option<usize>,

    pub(crate) cancel: &'a dyn Fn(usize),

    pub(crate) new_context: &'a dyn Fn(&str, Level, u64) -> &Context,
}

impl<'a> Context<'a> {
    /// 下单。
    ///
    /// * `side` 订单方向。
    /// * `price` 订单价格，0 表示市价，其他表示限价。
    /// * `size` 委托数量，单位 USDT，如果交易产品是合约，则会自动换算成张，0 表示由 [`Config`] 设置。
    /// * `stop_profit` 止盈价格，0 表示由 [`Config`] 设置。
    /// * `stop_loss` 止损价格，0 表示由 [`Config`] 设置。
    /// * `return` 订单 id。
    pub fn order(
        &self,
        side: Side,
        price: Unit,
        size: Unit,
        stop_profit: Unit,
        stop_loss: Unit,
    ) -> Option<usize> {
        (self.order)(side, price, size, stop_profit, stop_loss)
    }

    /// 下单。
    ///
    /// * `side` 订单方向。
    /// * `price` 订单价格，0 表示市价，其他表示限价。
    /// * `size` 委托数量，单位 USDT，如果交易产品是合约，则会自动换算成张，0 表示由 [`Config`] 设置。
    /// * `stop_profit` 止盈价格，0 表示由 [`Config`] 设置。
    /// * `stop_loss` 止损价格，0 表示由 [`Config`] 设置。
    /// * `return` 订单 id。
    pub fn order_quantity(
        &self,
        side: Side,
        price: f64,
        size: f64,
        stop_profit: f64,
        stop_loss: f64,
    ) -> Option<usize> {
        (self.order)(
            side,
            Quantity(price),
            Quantity(size),
            Quantity(stop_profit),
            Quantity(stop_loss),
        )
    }

    /// 下单。
    ///
    /// * `side` 订单方向。
    /// * `price` 订单价格，0 表示市价，其他表示限价。
    /// * `size` 委托比例，单位 USDT，如果交易产品是合约，则会自动换算成张，0 表示由 [`Config`] 设置。
    /// * `stop_profit` 止盈比例，0 表示由 [`Config`] 设置。
    /// * `stop_loss` 止损比例，0 表示由 [`Config`] 设置。
    /// * `return` 订单 id。
    pub fn order_proportion(
        &self,
        side: Side,
        price: f64,
        size: f64,
        stop_profit: f64,
        stop_loss: f64,
    ) -> Option<usize> {
        (self.order)(
            side,
            Proportion(price),
            Proportion(size),
            Proportion(stop_profit),
            Proportion(stop_loss),
        )
    }

    /// 撤单。
    ///
    /// * `side` 订单 id。
    pub fn cancel(&self, id: usize) {
        (self.cancel)(id)
    }

    /// 创建新的上下文环境，继承当前的上下文变量表。
    ///
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `level` 时间级别。
    /// * `time` 获取这个时间之前的数据，0 表示获取最近的数据。
    /// * `return` 上下文环境。
    pub fn new_context(&'a self, product: &'a str, level: Level, time: u64) -> &Context {
        (self.new_context)(product, level, time)
    }
}

impl<'a> std::ops::Index<&'static str> for Context<'a> {
    type Output = Value;

    fn index(&self, index: &'static str) -> &Self::Output {
        debug_assert!(self.variable.contains_key(index), "变量不存在: {}", index);
        unsafe { self.variable.get(index).unwrap_unchecked() }
    }
}

impl<'a> std::ops::IndexMut<&'static str> for Context<'a> {
    fn index_mut(&mut self, index: &'static str) -> &mut Self::Output {
        debug_assert!(self.variable.contains_key(index), "变量不存在: {}", index);
        unsafe { self.variable.get_mut(index).unwrap_unchecked() }
    }
}

/// 数量或比例。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Quantity(f64),
    Proportion(f64),
    Zero,
}

impl Unit {
    pub fn is_zero(&self) -> bool {
        match *self {
            Quantity(v) => v == 0.0,
            Proportion(v) => v == 0.0,
            Zero => true,
        }
    }
}

pub use Unit::Proportion;
pub use Unit::Quantity;
pub use Unit::Zero;

/// 交易配置，参数可以不设置，这取决于你的策略。
/// 例如，你的策略需要下单，但没有设置 `initial_margin` 和 `margin` 属性，则下单失败。
pub struct Config {
    pub(crate) initial_margin: f64,
    pub(crate) isolated: bool,
    pub(crate) lever: u32,
    pub(crate) fee: f64,
    pub(crate) deviation: f64,
    pub(crate) margin: Option<Unit>,
    pub(crate) max_margin: Option<Unit>,
    pub(crate) stop_profit: Option<Unit>,
    pub(crate) stop_loss: Option<Unit>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            initial_margin: 0.0,
            isolated: false,
            lever: 1,
            fee: 0.0,
            deviation: 0.0,
            margin: None,
            max_margin: None,
            stop_profit: None,
            stop_loss: None,
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

    /// 滑点。
    pub fn deviation(mut self, value: f64) -> Self {
        self.deviation = value;
        self
    }

    /// 每次开单投入的保证金。
    pub fn margin(mut self, value: Unit) -> Self {
        self.margin = Some(value);
        self
    }

    /// 最大投入的保证金数量，超过后将开单失败。
    pub fn max_margin(mut self, value: Unit) -> Self {
        self.max_margin = Some(value);
        self
    }

    /// 单笔止盈数量。
    pub fn stop_profit(mut self, value: Unit) -> Self {
        self.stop_profit = Some(value);
        self
    }

    /// 单笔止损数量。
    pub fn stop_loss(mut self, value: Unit) -> Self {
        self.stop_loss = Some(value);
        self
    }

    /// 每次开单投入的保证金。
    pub fn margin_quantity(mut self, value: f64) -> Self {
        self.margin = Some(Unit::Quantity(value));
        self
    }

    /// 每次开单投入的保证金比例。
    pub fn margin_proportion(mut self, value: f64) -> Self {
        self.margin = Some(Unit::Proportion(value));
        self
    }

    /// 最大投入的保证金数量，超过后将开单失败。
    pub fn max_margin_quantity(mut self, value: f64) -> Self {
        self.max_margin = Some(Unit::Quantity(value));
        self
    }

    /// 最大保证金比例，超过后将开单失败。
    pub fn max_margin_proportion(mut self, value: f64) -> Self {
        self.max_margin = Some(Unit::Proportion(value));
        self
    }

    /// 单笔止盈数量。
    pub fn stop_profit_quantity(mut self, value: f64) -> Self {
        self.stop_profit = Some(Unit::Quantity(value));
        self
    }

    /// 单笔止盈比例。
    pub fn stop_profit_proportion(mut self, value: f64) -> Self {
        self.stop_profit = Some(Unit::Proportion(value));
        self
    }

    /// 单笔止损数量。
    pub fn stop_loss_quantity(mut self, value: f64) -> Self {
        self.stop_loss = Some(Unit::Quantity(value));
        self
    }

    /// 单笔止损比例。
    pub fn stop_loss_proportion(mut self, value: f64) -> Self {
        self.stop_loss = Some(Unit::Proportion(value));
        self
    }
}

/// 交易所。
#[async_trait::async_trait]
pub trait Bourse {
    /// 获取 K 线最新价格。
    ///
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `level` 时间级别。
    /// * `time` 获取这个时间之前的数据，0 表示获取最近的数据。
    /// * `return` K 线数组。
    async fn get_k<S>(&self, product: S, level: Level, time: u64) -> anyhow::Result<Vec<K>>
    where
        S: AsRef<str>,
        S: core::marker::Send;

    /// 获取 K 线标记价格。
    ///
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `level` 时间级别。
    /// * `time` 获取这个时间之前的数据，0 表示获取最近的数据。
    /// * `return` K 线数组。
    async fn get_k_mark<S>(&self, product: S, level: Level, time: u64) -> anyhow::Result<Vec<K>>
    where
        S: AsRef<str>,
        S: core::marker::Send;

    /// 获取最小交易数量。
    ///
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    async fn get_min_unit<S>(&self, product: S) -> anyhow::Result<f64>
    where
        S: AsRef<str>,
        S: core::marker::Send;
}

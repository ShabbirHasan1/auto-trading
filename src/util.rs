use crate::*;

pub fn yield_map<'a, F>(source: &'a Source, f: F) -> impl Iterator<Item = f64> + 'a
where
    F: FnMut(&Source) -> f64 + 'a,
{
    source
        .into_iter()
        .enumerate()
        .map(|v| &source[v.0..])
        .map(f)
}

pub fn yield_nan<'a, F>(source: &'a Source, mut f: F) -> f64
where
    F: FnMut(f64, &Source) -> f64 + 'a,
{
    fn inner<'a, F>(source: &'a Source, f: &mut F) -> f64
    where
        F: FnMut(f64, &Source) -> f64 + 'a,
    {
        if source[1].is_nan() {
            f(f64::NAN, source)
        } else {
            let prev = inner(&source[1..], f);
            f(prev, source)
        }
    }

    inner(source, &mut f)
}

pub fn yield_nan_iter<'a, S, F>(source: S, mut f: F) -> f64
where
    S: IntoIterator<Item = f64>,
    F: FnMut(f64, &Source) -> f64 + 'a,
{
    fn inner<'a, S, F>(iter: S, source: &'a Source, f: &mut F) -> f64
    where
        S: IntoIterator<Item = f64>,
        F: FnMut(f64, &Source) -> f64 + 'a,
    {
        if source[1].is_nan() {
            f(f64::NAN, source)
        } else {
            let mut iter = iter.into_iter();
            let new = iter.next().unwrap_or(f64::NAN);
            let array = [source[1], new];
            let new_source = Source::new(array);
            let prev = inner(iter, new_source, f);
            f(prev, source)
        }
    }

    let mut iter = source.into_iter();
    let new = iter.next().unwrap_or(f64::NAN);
    let prev = iter.next().unwrap_or(f64::NAN);
    let array = [new, prev];
    let new_source = Source::new(array);

    inner(iter, new_source, &mut f)
}

pub fn yield_fold<'a, S, F>(source: S, mut f: F) -> f64
where
    S: IntoIterator<Item = f64>,
    F: FnMut(f64, &Source) -> f64 + 'a,
{
    let mut source = source.into_iter();

    let mut last = source.next().unwrap_or(f64::NAN);

    loop {
        let second = source.next().unwrap_or(f64::NAN);

        if second.is_nan() {
            return last;
        }

        last = f(last, Source::new([second, last]));
    }
}

pub fn highest(source: &Source, length: usize) -> f64 {
    if source.len() < length {
        return f64::NAN;
    }

    source
        .into_iter()
        .take(3)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap()
}

pub fn lowest(source: &Source, length: usize) -> f64 {
    if source.len() < length {
        return f64::NAN;
    }

    source
        .into_iter()
        .take(3)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap()
}

pub fn sma(source: &Source, length: usize) -> f64 {
    if source.len() < length {
        return f64::NAN;
    }

    source.iter().take(length).sum::<f64>() / length as f64
}

pub fn ema(source: &Source, length: usize) -> f64 {
    let alpha = 2.0 / (length + 1) as f64;

    yield_nan(source, |prev, source| {
        if prev.is_nan() {
            source[0]
        } else {
            alpha * source + (1.0 - alpha) * prev
        }
    })
}

pub fn ema_iter<S>(source: S, length: usize) -> f64
where
    S: IntoIterator<Item = f64>,
{
    let alpha = 2.0 / (length + 1) as f64;

    yield_nan_iter(source, |prev, source| {
        if prev.is_nan() {
            source[0]
        } else {
            alpha * source + (1.0 - alpha) * prev
        }
    })
}

pub fn cci(source: &Source, length: usize) -> f64 {
    if source.len() < length {
        return f64::NAN;
    }

    let ma = sma(source, length);
    let sum = yield_map(&source[..length], |v| (v - ma).abs()).sum::<f64>();
    (source - ma) / (0.015 * (sum / length as f64))
}

pub fn macd(
    source: &Source,
    short_length: usize,
    long_length: usize,
    dea_length: usize,
) -> (f64, f64, f64) {
    let iter = yield_map(source, |v| ema(v, short_length) - ema(v, long_length));
    let dif = ema(source, short_length) - ema(source, long_length);
    let dea = ema_iter(iter, dea_length);
    let macd = (dif - dea) * 2.0;
    (dif, dea, macd)
}

/// 时间戳转换到本地时间文本。
///
/// * `value` 时间戳。
/// * `return` 本地时间文本。
pub fn time_to_string(value: u64) -> String {
    chrono::NaiveDateTime::from_timestamp_millis(value as i64)
        .unwrap()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

/// 获取指定范围的 k 线数据。
/// 新的数据在前面。
///
/// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
/// * `level` 时间级别。
/// * `range` 时间范围，0 表示获取所有数据，a..b 表示时间戳 a 到时间戳 b 范围之内的数据，
/// * `return` K 线数组。
pub async fn get_k_range<E, S, T>(
    exchange: &E,
    product: S,
    level: Level,
    range: T,
) -> anyhow::Result<Vec<K>>
where
    E: Exchange,
    S: AsRef<str>,
    T: Into<TimeRange>,
{
    let product = product.as_ref();
    let range = range.into();

    let mut result = Vec::new();

    if range.start == 0 && range.end == 0 {
        let mut time = 0;

        loop {
            let v = exchange.get_k(product, level, time).await?;

            if let Some(k) = v.last() {
                time = k.time;
                result.extend(v);
            } else {
                break;
            }
        }

        return Ok(result);
    }

    let mut end = range.end;

    if end == u64::MAX - 1 {
        end = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
    }

    loop {
        let v = exchange.get_k(product, level, end).await?;

        if let Some(k) = v.last() {
            if k.time < range.start {
                for i in v {
                    if i.time >= range.start {
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

/// 交易产品映射。
/// BTC-USDT <-> BTCUSDT。
/// BTC-USDT-SWAP <-> BTCUSDTSWAP。
///
/// * `value` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
/// * `return` 映射值。
pub fn product_mapping<S>(value: S) -> std::borrow::Cow<'static, str>
where
    S: AsRef<str>,
{
    let result = match value.as_ref() {
        "MDT-USDT" => "MDTUSDT",
        "MDT-USDT-SWAP" => "MDTUSDTSWAP",
        "KNC-USDT" => "KNCUSDT",
        "KNC-USDT-SWAP" => "KNCUSDTSWAP",
        "CVX-USDT" => "CVXUSDT",
        "CVX-USDT-SWAP" => "CVXUSDTSWAP",
        "AGLD-USDT" => "AGLDUSDT",
        "AGLD-USDT-SWAP" => "AGLDUSDT",
        "IOTA-USDT" => "IOTAUSDT",
        "IOTA-USDT-SWAP" => "IOTAUSDT",
        "QTUM-USDT" => "QTUMUSDT",
        "QTUM-USDT-SWAP" => "QTUMUSDT",
        "AXS-USDT" => "AXSUSDT",
        "AXS-USDT-SWAP" => "AXSUSDT",
        "LQTY-USDT" => "LQTYUSDT",
        "LQTY-USDT-SWAP" => "LQTYUSDT",
        "ETC-USDT" => "ETCUSDT",
        "ETC-USDT-SWAP" => "ETCUSDT",
        "ASTR-USDT" => "ASTRUSDT",
        "ASTR-USDT-SWAP" => "ASTRUSDT",
        "BCH-USDT" => "BCHUSDT",
        "BCH-USDT-SWAP" => "BCHUSDT",
        "APT-USDT" => "APTUSDT",
        "APT-USDT-SWAP" => "APTUSDT",
        "TRX-USDT" => "TRXUSDT",
        "TRX-USDT-SWAP" => "TRXUSDT",
        "CELR-USDT" => "CELRUSDT",
        "CELR-USDT-SWAP" => "CELRUSDT",
        "CELO-USDT" => "CELOUSDT",
        "CELO-USDT-SWAP" => "CELOUSDT",
        "SAND-USDT" => "SANDUSDT",
        "SAND-USDT-SWAP" => "SANDUSDT",
        "KLAY-USDT" => "KLAYUSDT",
        "KLAY-USDT-SWAP" => "KLAYUSDT",
        "T-USDT" => "TUSDT",
        "T-USDT-SWAP" => "TUSDT",
        "FLM-USDT" => "FLMUSDT",
        "FLM-USDT-SWAP" => "FLMUSDT",
        "RAY-USDT" => "RAYUSDT",
        "RAY-USDT-SWAP" => "RAYUSDT",
        "SOL-USDT" => "SOLUSDT",
        "SOL-USDT-SWAP" => "SOLUSDT",
        "API3-USDT" => "API3USDT",
        "API3-USDT-SWAP" => "API3USDT",
        "YFI-USDT" => "YFIUSDT",
        "YFI-USDT-SWAP" => "YFIUSDT",
        "LDO-USDT" => "LDOUSDT",
        "LDO-USDT-SWAP" => "LDOUSDT",
        "NMR-USDT" => "NMRUSDT",
        "NMR-USDT-SWAP" => "NMRUSDT",
        "AAVE-USDT" => "AAVEUSDT",
        "AAVE-USDT-SWAP" => "AAVEUSDT",
        "TRB-USDT" => "TRBUSDT",
        "TRB-USDT-SWAP" => "TRBUSDT",
        "MATIC-USDT" => "MATICUSDT",
        "MATIC-USDT-SWAP" => "MATICUSDT",
        "DOGE-USDT" => "DOGEUSDT",
        "DOGE-USDT-SWAP" => "DOGEUSDT",
        "XRP-USDT" => "XRPUSDT",
        "XRP-USDT-SWAP" => "XRPUSDT",
        "ENS-USDT" => "ENSUSDT",
        "ENS-USDT-SWAP" => "ENSUSDT",
        "LPT-USDT" => "LPTUSDT",
        "LPT-USDT-SWAP" => "LPTUSDT",
        "BNB-USDT" => "BNBUSDT",
        "BNB-USDT-SWAP" => "BNBUSDT",
        "SPELL-USDT" => "SPELLUSDT",
        "SPELL-USDT-SWAP" => "SPELLUSDT",
        "CRV-USDT" => "CRVUSDT",
        "CRV-USDT-SWAP" => "CRVUSDT",
        "ARB-USDT" => "ARBUSDT",
        "ARB-USDT-SWAP" => "ARBUSDT",
        "EGLD-USDT" => "EGLDUSDT",
        "EGLD-USDT-SWAP" => "EGLDUSDT",
        "NEO-USDT" => "NEOUSDT",
        "NEO-USDT-SWAP" => "NEOUSDT",
        "CHZ-USDT" => "CHZUSDT",
        "CHZ-USDT-SWAP" => "CHZUSDT",
        "USDC-USDT" => "USDCUSDT",
        "USDC-USDT-SWAP" => "USDCUSDT",
        "DOT-USDT" => "DOTUSDT",
        "DOT-USDT-SWAP" => "DOTUSDT",
        "MINA-USDT" => "MINAUSDT",
        "MINA-USDT-SWAP" => "MINAUSDT",
        "EOS-USDT" => "EOSUSDT",
        "EOS-USDT-SWAP" => "EOSUSDT",
        "GMX-USDT" => "GMXUSDT",
        "GMX-USDT-SWAP" => "GMXUSDT",
        "GRT-USDT" => "GRTUSDT",
        "GRT-USDT-SWAP" => "GRTUSDT",
        "SKL-USDT" => "SKLUSDT",
        "SKL-USDT-SWAP" => "SKLUSDT",
        "IOST-USDT" => "IOSTUSDT",
        "IOST-USDT-SWAP" => "IOSTUSDT",
        "RDNT-USDT" => "RDNTUSDT",
        "RDNT-USDT-SWAP" => "RDNTUSDT",
        "BTC-USDT" => "BTCUSDT",
        "BTC-USDT-SWAP" => "BTCUSDT",
        "WOO-USDT" => "WOOUSDT",
        "WOO-USDT-SWAP" => "WOOUSDT",
        "ACH-USDT" => "ACHUSDT",
        "ACH-USDT-SWAP" => "ACHUSDT",
        "APE-USDT" => "APEUSDT",
        "APE-USDT-SWAP" => "APEUSDT",
        "ID-USDT" => "IDUSDT",
        "ID-USDT-SWAP" => "IDUSDT",
        "ADA-USDT" => "ADAUSDT",
        "ADA-USDT-SWAP" => "ADAUSDT",
        "HNT-USDT" => "HNTUSDT",
        "HNT-USDT-SWAP" => "HNTUSDT",
        "ALPHA-USDT" => "ALPHAUSDT",
        "ALPHA-USDT-SWAP" => "ALPHAUSDT",
        "CFX-USDT" => "CFXUSDT",
        "CFX-USDT-SWAP" => "CFXUSDT",
        "SRM-USDT" => "SRMUSDT",
        "SRM-USDT-SWAP" => "SRMUSDT",
        "UNI-USDT" => "UNIUSDT",
        "UNI-USDT-SWAP" => "UNIUSDT",
        "THETA-USDT" => "THETAUSDT",
        "THETA-USDT-SWAP" => "THETAUSDT",
        "HBAR-USDT" => "HBARUSDT",
        "HBAR-USDT-SWAP" => "HBARUSDT",
        "ZEC-USDT" => "ZECUSDT",
        "ZEC-USDT-SWAP" => "ZECUSDT",
        "SUSHI-USDT" => "SUSHIUSDT",
        "SUSHI-USDT-SWAP" => "SUSHIUSDT",
        "LTC-USDT" => "LTCUSDT",
        "LTC-USDT-SWAP" => "LTCUSDT",
        "ICX-USDT" => "ICXUSDT",
        "ICX-USDT-SWAP" => "ICXUSDT",
        "LINK-USDT" => "LINKUSDT",
        "LINK-USDT-SWAP" => "LINKUSDT",
        "XTZ-USDT" => "XTZUSDT",
        "XTZ-USDT-SWAP" => "XTZUSDT",
        "RVN-USDT" => "RVNUSDT",
        "RVN-USDT-SWAP" => "RVNUSDT",
        "WLD-USDT" => "WLDUSDT",
        "WLD-USDT-SWAP" => "WLDUSDT",
        "OP-USDT" => "OPUSDT",
        "OP-USDT-SWAP" => "OPUSDT",
        "REN-USDT" => "RENUSDT",
        "REN-USDT-SWAP" => "RENUSDT",
        "BLUR-USDT" => "BLURUSDT",
        "BLUR-USDT-SWAP" => "BLURUSDT",
        "SUI-USDT" => "SUIUSDT",
        "SUI-USDT-SWAP" => "SUIUSDT",
        "ICP-USDT" => "ICPUSDT",
        "ICP-USDT-SWAP" => "ICPUSDT",
        "XMR-USDT" => "XMRUSDT",
        "XMR-USDT-SWAP" => "XMRUSDT",
        "ZEN-USDT" => "ZENUSDT",
        "ZEN-USDT-SWAP" => "ZENUSDT",
        "FTM-USDT" => "FTMUSDT",
        "FTM-USDT-SWAP" => "FTMUSDT",
        "MAGIC-USDT" => "MAGICUSDT",
        "MAGIC-USDT-SWAP" => "MAGICUSDT",
        "DGB-USDT" => "DGBUSDT",
        "DGB-USDT-SWAP" => "DGBUSDT",
        "LRC-USDT" => "LRCUSDT",
        "LRC-USDT-SWAP" => "LRCUSDT",
        "DYDX-USDT" => "DYDXUSDT",
        "DYDX-USDT-SWAP" => "DYDXUSDT",
        "ZRX-USDT" => "ZRXUSDT",
        "ZRX-USDT-SWAP" => "ZRXUSDT",
        "SC-USDT" => "SCUSDT",
        "SC-USDT-SWAP" => "SCUSDT",
        "FIL-USDT" => "FILUSDT",
        "FIL-USDT-SWAP" => "FILUSDT",
        "RSR-USDT" => "RSRUSDT",
        "RSR-USDT-SWAP" => "RSRUSDT",
        "ETH-BTC" => "ETHBTC",
        "ETH-BTC-SWAP" => "ETHBTC",
        "ONT-USDT" => "ONTUSDT",
        "ONT-USDT-SWAP" => "ONTUSDT",
        "FXS-USDT" => "FXSUSDT",
        "FXS-USDT-SWAP" => "FXSUSDT",
        "UMA-USDT" => "UMAUSDT",
        "UMA-USDT-SWAP" => "UMAUSDT",
        "AR-USDT" => "ARUSDT",
        "AR-USDT-SWAP" => "ARUSDT",
        "BAND-USDT" => "BANDUSDT",
        "BAND-USDT-SWAP" => "BANDUSDT",
        "XLM-USDT" => "XLMUSDT",
        "XLM-USDT-SWAP" => "XLMUSDT",
        "SNX-USDT" => "SNXUSDT",
        "SNX-USDT-SWAP" => "SNXUSDT",
        "ATOM-USDT" => "ATOMUSDT",
        "ATOM-USDT-SWAP" => "ATOMUSDT",
        "BAT-USDT" => "BATUSDT",
        "BAT-USDT-SWAP" => "BATUSDT",
        "MANA-USDT" => "MANAUSDT",
        "MANA-USDT-SWAP" => "MANAUSDT",
        "CVC-USDT" => "CVCUSDT",
        "CVC-USDT-SWAP" => "CVCUSDT",
        "XEM-USDT" => "XEMUSDT",
        "XEM-USDT-SWAP" => "XEMUSDT",
        "SSV-USDT" => "SSVUSDT",
        "SSV-USDT-SWAP" => "SSVUSDT",
        "KSM-USDT" => "KSMUSDT",
        "KSM-USDT-SWAP" => "KSMUSDT",
        "JOE-USDT" => "JOEUSDT",
        "JOE-USDT-SWAP" => "JOEUSDT",
        "ETH-USDT" => "ETHUSDT",
        "ETH-USDT-SWAP" => "ETHUSDT",
        "STORJ-USDT" => "STORJUSDT",
        "STORJ-USDT-SWAP" => "STORJUSDT",
        "GMT-USDT" => "GMTUSDT",
        "GMT-USDT-SWAP" => "GMTUSDT",
        "OMG-USDT" => "OMGUSDT",
        "OMG-USDT-SWAP" => "OMGUSDT",
        "PEOPLE-USDT" => "PEOPLEUSDT",
        "PEOPLE-USDT-SWAP" => "PEOPLEUSDT",
        "BAL-USDT" => "BALUSDT",
        "BAL-USDT-SWAP" => "BALUSDT",
        "ZIL-USDT" => "ZILUSDT",
        "ZIL-USDT-SWAP" => "ZILUSDT",
        "FLOW-USDT" => "FLOWUSDT",
        "FLOW-USDT-SWAP" => "FLOWUSDT",
        "IMX-USDT" => "IMXUSDT",
        "IMX-USDT-SWAP" => "IMXUSDT",
        "COMP-USDT" => "COMPUSDT",
        "COMP-USDT-SWAP" => "COMPUSDT",
        "ALGO-USDT" => "ALGOUSDT",
        "ALGO-USDT-SWAP" => "ALGOUSDT",
        "WAVES-USDT" => "WAVESUSDT",
        "WAVES-USDT-SWAP" => "WAVESUSDT",
        "DASH-USDT" => "DASHUSDT",
        "DASH-USDT-SWAP" => "DASHUSDT",
        "ENJ-USDT" => "ENJUSDT",
        "ENJ-USDT-SWAP" => "ENJUSDT",
        "1INCH-USDT" => "1INCHUSDT",
        "1INCH-USDT-SWAP" => "1INCHUSDT",
        "PERP-USDT" => "PERPUSDT",
        "PERP-USDT-SWAP" => "PERPUSDT",
        "NEAR-USDT" => "NEARUSDT",
        "NEAR-USDT-SWAP" => "NEARUSDT",
        "ANT-USDT" => "ANTUSDT",
        "ANT-USDT-SWAP" => "ANTUSDT",
        "GAL-USDT" => "GALUSDT",
        "GAL-USDT-SWAP" => "GALUSDT",
        "ONE-USDT" => "ONEUSDT",
        "ONE-USDT-SWAP" => "ONEUSDT",
        "MKR-USDT" => "MKRUSDT",
        "MKR-USDT-SWAP" => "MKRUSDT",
        "GALA-USDT" => "GALAUSDT",
        "GALA-USDT-SWAP" => "GALAUSDT",
        "AVAX-USDT" => "AVAXUSDT",
        "AVAX-USDT-SWAP" => "AVAXUSDT",
        "MASK-USDT" => "MASKUSDT",
        "MASK-USDT-SWAP" => "MASKUSDT",
        "STX-USDT" => "STXUSDT",
        "STX-USDT-SWAP" => "STXUSDT",
        // ================================
        "MDTUSDT" => "MDT-USDT",
        "MDTUSDTSWAP" => "MDT-USDT-SWAP",
        "KNCUSDT" => "KNC-USDT",
        "KNCUSDTSWAP" => "KNC-USDT-SWAP",
        "CVXUSDT" => "CVX-USDT",
        "CVXUSDTSWAP" => "CVX-USDT-SWAP",
        "AGLDUSDT" => "AGLD-USDT",
        "AGLDUSDTSWAP" => "AGLD-USDT-SWAP",
        "IOTAUSDT" => "IOTA-USDT",
        "IOTAUSDTSWAP" => "IOTA-USDT-SWAP",
        "QTUMUSDT" => "QTUM-USDT",
        "QTUMUSDTSWAP" => "QTUM-USDT-SWAP",
        "AXSUSDT" => "AXS-USDT",
        "AXSUSDTSWAP" => "AXS-USDT-SWAP",
        "LQTYUSDT" => "LQTY-USDT",
        "LQTYUSDTSWAP" => "LQTY-USDT-SWAP",
        "ETCUSDT" => "ETC-USDT",
        "ETCUSDTSWAP" => "ETC-USDT-SWAP",
        "ASTRUSDT" => "ASTR-USDT",
        "ASTRUSDTSWAP" => "ASTR-USDT-SWAP",
        "BCHUSDT" => "BCH-USDT",
        "BCHUSDTSWAP" => "BCH-USDT-SWAP",
        "APTUSDT" => "APT-USDT",
        "APTUSDTSWAP" => "APT-USDT-SWAP",
        "TRXUSDT" => "TRX-USDT",
        "TRXUSDTSWAP" => "TRX-USDT-SWAP",
        "CELRUSDT" => "CELR-USDT",
        "CELRUSDTSWAP" => "CELR-USDT-SWAP",
        "CELOUSDT" => "CELO-USDT",
        "CELOUSDTSWAP" => "CELO-USDT-SWAP",
        "SANDUSDT" => "SAND-USDT",
        "SANDUSDTSWAP" => "SAND-USDT-SWAP",
        "KLAYUSDT" => "KLAY-USDT",
        "KLAYUSDTSWAP" => "KLAY-USDT-SWAP",
        "TUSDT" => "T-USDT",
        "TUSDTSWAP" => "T-USDT-SWAP",
        "FLMUSDT" => "FLM-USDT",
        "FLMUSDTSWAP" => "FLM-USDT-SWAP",
        "RAYUSDT" => "RAY-USDT",
        "RAYUSDTSWAP" => "RAY-USDT-SWAP",
        "SOLUSDT" => "SOL-USDT",
        "SOLUSDTSWAP" => "SOL-USDT-SWAP",
        "API3USDT" => "API3-USDT",
        "API3USDTSWAP" => "API3-USDT-SWAP",
        "YFIUSDT" => "YFI-USDT",
        "YFIUSDTSWAP" => "YFI-USDT-SWAP",
        "LDOUSDT" => "LDO-USDT",
        "LDOUSDTSWAP" => "LDO-USDT-SWAP",
        "NMRUSDT" => "NMR-USDT",
        "NMRUSDTSWAP" => "NMR-USDT-SWAP",
        "AAVEUSDT" => "AAVE-USDT",
        "AAVEUSDTSWAP" => "AAVE-USDT-SWAP",
        "TRBUSDT" => "TRB-USDT",
        "TRBUSDTSWAP" => "TRB-USDT-SWAP",
        "MATICUSDT" => "MATIC-USDT",
        "MATICUSDTSWAP" => "MATIC-USDT-SWAP",
        "DOGEUSDT" => "DOGE-USDT",
        "DOGEUSDTSWAP" => "DOGE-USDT-SWAP",
        "XRPUSDT" => "XRP-USDT",
        "XRPUSDTSWAP" => "XRP-USDT-SWAP",
        "ENSUSDT" => "ENS-USDT",
        "ENSUSDTSWAP" => "ENS-USDT-SWAP",
        "LPTUSDT" => "LPT-USDT",
        "LPTUSDTSWAP" => "LPT-USDT-SWAP",
        "BNBUSDT" => "BNB-USDT",
        "BNBUSDTSWAP" => "BNB-USDT-SWAP",
        "SPELLUSDT" => "SPELL-USDT",
        "SPELLUSDTSWAP" => "SPELL-USDT-SWAP",
        "CRVUSDT" => "CRV-USDT",
        "CRVUSDTSWAP" => "CRV-USDT-SWAP",
        "ARBUSDT" => "ARB-USDT",
        "ARBUSDTSWAP" => "ARB-USDT-SWAP",
        "EGLDUSDT" => "EGLD-USDT",
        "EGLDUSDTSWAP" => "EGLD-USDT-SWAP",
        "NEOUSDT" => "NEO-USDT",
        "NEOUSDTSWAP" => "NEO-USDT-SWAP",
        "CHZUSDT" => "CHZ-USDT",
        "CHZUSDTSWAP" => "CHZ-USDT-SWAP",
        "USDCUSDT" => "USDC-USDT",
        "USDCUSDTSWAP" => "USDC-USDT-SWAP",
        "DOTUSDT" => "DOT-USDT",
        "DOTUSDTSWAP" => "DOT-USDT-SWAP",
        "MINAUSDT" => "MINA-USDT",
        "MINAUSDTSWAP" => "MINA-USDT-SWAP",
        "EOSUSDT" => "EOS-USDT",
        "EOSUSDTSWAP" => "EOS-USDT-SWAP",
        "GMXUSDT" => "GMX-USDT",
        "GMXUSDTSWAP" => "GMX-USDT-SWAP",
        "GRTUSDT" => "GRT-USDT",
        "GRTUSDTSWAP" => "GRT-USDT-SWAP",
        "SKLUSDT" => "SKL-USDT",
        "SKLUSDTSWAP" => "SKL-USDT-SWAP",
        "IOSTUSDT" => "IOST-USDT",
        "IOSTUSDTSWAP" => "IOST-USDT-SWAP",
        "RDNTUSDT" => "RDNT-USDT",
        "RDNTUSDTSWAP" => "RDNT-USDT-SWAP",
        "BTCUSDT" => "BTC-USDT",
        "BTCUSDTSWAP" => "BTC-USDT-SWAP",
        "WOOUSDT" => "WOO-USDT",
        "WOOUSDTSWAP" => "WOO-USDT-SWAP",
        "ACHUSDT" => "ACH-USDT",
        "ACHUSDTSWAP" => "ACH-USDT-SWAP",
        "APEUSDT" => "APE-USDT",
        "APEUSDTSWAP" => "APE-USDT-SWAP",
        "IDUSDT" => "ID-USDT",
        "IDUSDTSWAP" => "ID-USDT-SWAP",
        "ADAUSDT" => "ADA-USDT",
        "ADAUSDTSWAP" => "ADA-USDT-SWAP",
        "HNTUSDT" => "HNT-USDT",
        "HNTUSDTSWAP" => "HNT-USDT-SWAP",
        "ALPHAUSDT" => "ALPHA-USDT",
        "ALPHAUSDTSWAP" => "ALPHA-USDT-SWAP",
        "CFXUSDT" => "CFX-USDT",
        "CFXUSDTSWAP" => "CFX-USDT-SWAP",
        "SRMUSDT" => "SRM-USDT",
        "SRMUSDTSWAP" => "SRM-USDT-SWAP",
        "UNIUSDT" => "UNI-USDT",
        "UNIUSDTSWAP" => "UNI-USDT-SWAP",
        "THETAUSDT" => "THETA-USDT",
        "THETAUSDTSWAP" => "THETA-USDT-SWAP",
        "HBARUSDT" => "HBAR-USDT",
        "HBARUSDTSWAP" => "HBAR-USDT-SWAP",
        "ZECUSDT" => "ZEC-USDT",
        "ZECUSDTSWAP" => "ZEC-USDT-SWAP",
        "SUSHIUSDT" => "SUSHI-USDT",
        "SUSHIUSDTSWAP" => "SUSHI-USDT-SWAP",
        "LTCUSDT" => "LTC-USDT",
        "LTCUSDTSWAP" => "LTC-USDT-SWAP",
        "ICXUSDT" => "ICX-USDT",
        "ICXUSDTSWAP" => "ICX-USDT-SWAP",
        "LINKUSDT" => "LINK-USDT",
        "LINKUSDTSWAP" => "LINK-USDT-SWAP",
        "XTZUSDT" => "XTZ-USDT",
        "XTZUSDTSWAP" => "XTZ-USDT-SWAP",
        "RVNUSDT" => "RVN-USDT",
        "RVNUSDTSWAP" => "RVN-USDT-SWAP",
        "WLDUSDT" => "WLD-USDT",
        "WLDUSDTSWAP" => "WLD-USDT-SWAP",
        "OPUSDT" => "OP-USDT",
        "OPUSDTSWAP" => "OP-USDT-SWAP",
        "RENUSDT" => "REN-USDT",
        "RENUSDTSWAP" => "REN-USDT-SWAP",
        "BLURUSDT" => "BLUR-USDT",
        "BLURUSDTSWAP" => "BLUR-USDT-SWAP",
        "SUIUSDT" => "SUI-USDT",
        "SUIUSDTSWAP" => "SUI-USDT-SWAP",
        "ICPUSDT" => "ICP-USDT",
        "ICPUSDTSWAP" => "ICP-USDT-SWAP",
        "XMRUSDT" => "XMR-USDT",
        "XMRUSDTSWAP" => "XMR-USDT-SWAP",
        "ZENUSDT" => "ZEN-USDT",
        "ZENUSDTSWAP" => "ZEN-USDT-SWAP",
        "FTMUSDT" => "FTM-USDT",
        "FTMUSDTSWAP" => "FTM-USDT-SWAP",
        "MAGICUSDT" => "MAGIC-USDT",
        "MAGICUSDTSWAP" => "MAGIC-USDT-SWAP",
        "DGBUSDT" => "DGB-USDT",
        "DGBUSDTSWAP" => "DGB-USDT-SWAP",
        "LRCUSDT" => "LRC-USDT",
        "LRCUSDTSWAP" => "LRC-USDT-SWAP",
        "DYDXUSDT" => "DYDX-USDT",
        "DYDXUSDTSWAP" => "DYDX-USDT-SWAP",
        "ZRXUSDT" => "ZRX-USDT",
        "ZRXUSDTSWAP" => "ZRX-USDT-SWAP",
        "SCUSDT" => "SC-USDT",
        "SCUSDTSWAP" => "SC-USDT-SWAP",
        "FILUSDT" => "FIL-USDT",
        "FILUSDTSWAP" => "FIL-USDT-SWAP",
        "RSRUSDT" => "RSR-USDT",
        "RSRUSDTSWAP" => "RSR-USDT-SWAP",
        "ETHBTC" => "ETH-BTC",
        "ETHBTCSWAP" => "ETH-BTC-SWAP",
        "ONTUSDT" => "ONT-USDT",
        "ONTUSDTSWAP" => "ONT-USDT-SWAP",
        "FXSUSDT" => "FXS-USDT",
        "FXSUSDTSWAP" => "FXS-USDT-SWAP",
        "UMAUSDT" => "UMA-USDT",
        "UMAUSDTSWAP" => "UMA-USDT-SWAP",
        "ARUSDT" => "AR-USDT",
        "ARUSDTSWAP" => "AR-USDT-SWAP",
        "BANDUSDT" => "BAND-USDT",
        "BANDUSDTSWAP" => "BAND-USDT-SWAP",
        "XLMUSDT" => "XLM-USDT",
        "XLMUSDTSWAP" => "XLM-USDT-SWAP",
        "SNXUSDT" => "SNX-USDT",
        "SNXUSDTSWAP" => "SNX-USDT-SWAP",
        "ATOMUSDT" => "ATOM-USDT",
        "ATOMUSDTSWAP" => "ATOM-USDT-SWAP",
        "BATUSDT" => "BAT-USDT",
        "BATUSDTSWAP" => "BAT-USDT-SWAP",
        "MANAUSDT" => "MANA-USDT",
        "MANAUSDTSWAP" => "MANA-USDT-SWAP",
        "CVCUSDT" => "CVC-USDT",
        "CVCUSDTSWAP" => "CVC-USDT-SWAP",
        "XEMUSDT" => "XEM-USDT",
        "XEMUSDTSWAP" => "XEM-USDT-SWAP",
        "SSVUSDT" => "SSV-USDT",
        "SSVUSDTSWAP" => "SSV-USDT-SWAP",
        "KSMUSDT" => "KSM-USDT",
        "KSMUSDTSWAP" => "KSM-USDT-SWAP",
        "JOEUSDT" => "JOE-USDT",
        "JOEUSDTSWAP" => "JOE-USDT-SWAP",
        "ETHUSDT" => "ETH-USDT",
        "ETHUSDTSWAP" => "ETH-USDT-SWAP",
        "STORJUSDT" => "STORJ-USDT",
        "STORJUSDTSWAP" => "STORJ-USDT-SWAP",
        "GMTUSDT" => "GMT-USDT",
        "GMTUSDTSWAP" => "GMT-USDT-SWAP",
        "OMGUSDT" => "OMG-USDT",
        "OMGUSDTSWAP" => "OMG-USDT-SWAP",
        "PEOPLEUSDT" => "PEOPLE-USDT",
        "PEOPLEUSDTSWAP" => "PEOPLE-USDT-SWAP",
        "BALUSDT" => "BAL-USDT",
        "BALUSDTSWAP" => "BAL-USDT-SWAP",
        "ZILUSDT" => "ZIL-USDT",
        "ZILUSDTSWAP" => "ZIL-USDT-SWAP",
        "FLOWUSDT" => "FLOW-USDT",
        "FLOWUSDTSWAP" => "FLOW-USDT-SWAP",
        "IMXUSDT" => "IMX-USDT",
        "IMXUSDTSWAP" => "IMX-USDT-SWAP",
        "COMPUSDT" => "COMP-USDT",
        "COMPUSDTSWAP" => "COMP-USDT-SWAP",
        "ALGOUSDT" => "ALGO-USDT",
        "ALGOUSDTSWAP" => "ALGO-USDT-SWAP",
        "WAVESUSDT" => "WAVES-USDT",
        "WAVESUSDTSWAP" => "WAVES-USDT-SWAP",
        "DASHUSDT" => "DASH-USDT",
        "DASHUSDTSWAP" => "DASH-USDT-SWAP",
        "ENJUSDT" => "ENJ-USDT",
        "ENJUSDTSWAP" => "ENJ-USDT-SWAP",
        "1INCHUSDT" => "1INCH-USDT",
        "1INCHUSDTSWAP" => "1INCH-USDT-SWAP",
        "PERPUSDT" => "PERP-USDT",
        "PERPUSDTSWAP" => "PERP-USDT-SWAP",
        "NEARUSDT" => "NEAR-USDT",
        "NEARUSDTSWAP" => "NEAR-USDT-SWAP",
        "ANTUSDT" => "ANT-USDT",
        "ANTUSDTSWAP" => "ANT-USDT-SWAP",
        "GALUSDT" => "GAL-USDT",
        "GALUSDTSWAP" => "GAL-USDT-SWAP",
        "ONEUSDT" => "ONE-USDT",
        "ONEUSDTSWAP" => "ONE-USDT-SWAP",
        "MKRUSDT" => "MKR-USDT",
        "MKRUSDTSWAP" => "MKR-USDT-SWAP",
        "GALAUSDT" => "GALA-USDT",
        "GALAUSDTSWAP" => "GALA-USDT-SWAP",
        "AVAXUSDT" => "AVAX-USDT",
        "AVAXUSDTSWAP" => "AVAX-USDT-SWAP",
        "MASKUSDT" => "MASK-USDT",
        "MASKUSDTSWAP" => "MASK-USDT-SWAP",
        "STXUSDT" => "STX-USDT",
        "STXUSDTSWAP" => "STX-USDT-SWAP",
        _ => "",
    };

    if result.is_empty() {
        std::borrow::Cow::Owned(if result.contains("-") {
            result.replace("-", "")
        } else if result.ends_with("SWAP") {
            result.replace("USDT", "-USDT-")
        } else {
            result.replace("USDT", "-USDT")
        })
    } else {
        std::borrow::Cow::Borrowed(result)
    }
}

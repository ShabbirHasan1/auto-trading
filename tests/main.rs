use auto_trading::*;

#[tokio::test]
async fn test_1() {
    let exchange = Okx::new().unwrap();

    let config = Config::new()
        .initial_margin(1000.0)
        .quantity(Unit::Quantity(0.01))
        .margin(Unit::Quantity(10.0))
        .lever(100)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004);

    let backtester = Backtester::new(exchange, config);

    let result = backtester
        .start(
            |cx| {
                if cx.position().is_none() {
                    if cci(cx.close, 20) <= -350.0 {
                        let result = cx.order(Side::BuyLong, 0.0);
                        println!(
                            "开仓委托结果 {} {} {:?}",
                            time_to_string(cx.time),
                            cx.close,
                            result
                        );
                    }
                } else {
                    if cci(cx.close, 20) >= 100.0 {
                        let result = cx.order(Side::BuySell, 0.0);
                        println!(
                            "平仓委托结果 {} {} {:?}",
                            time_to_string(cx.time),
                            cx.close,
                            result
                        );
                    }
                }
            },
            "BTC-USDT-SWAP",
            Level::Hour4,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
                - 1000 * 60 * 60 * 24 * 7 * 4..,
        )
        .await
        .unwrap();

    println!("历史仓位 {:#?}", result);
    println!("所有盈亏 {}", result.iter().map(|v| v.profit).sum::<f64>());
}

#[tokio::test]
async fn test_2() {
    // 使用 1 分钟的 k 线数据。
    let k = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-1m.json")).unwrap();

    let exchange = LocalExchange::new()
        .push("BTC-USDT-SWAP", Level::Minute1, k.clone(), 0.01, 0.0)
        .push(
            "BTC-USDT-SWAP",
            Level::Hour4,
            k_convert(k, Level::Hour4),
            0.01,
            0.0,
        );

    // Level::Minute1 -> Level::Hour4
    Backtester::new(exchange, Config::new())
        .start_amplifier(
            |cx| println!("{} {}", cx.time, time_to_string(cx.time)),
            "BTC-USDT-SWAP",
            Level::Minute1,
            Level::Hour4,
            0,
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn test_3() {
    println!("{}", time_to_string(1145141919810));
    println!("{}", string_to_time("2006-04-16 06:58:39"));
    println!(
        "{:?}",
        get_k_range(
            &Okx::new().unwrap(),
            "BTC-USDT-SWAP",
            Level::Hour4,
            1695212739000..1695644739000
        )
        .await
        .unwrap()
    );
}

#[test]
fn test_4() {
    let array = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let source = Source::new(&array);
    assert!(source == 1.0);
    assert!(source[2] == 3.0);
    assert!(source[10].is_nan());
    assert!((&source[1..4]) == &[2.0, 3.0, 4.0][..]);
    assert!((&source[10..]).len() == 0);
}

#[tokio::test]
async fn test_local() {
    // 使用本地交易所
    let exchange = LocalExchange::new().push(
        "BTC-USDT-SWAP",
        Level::Hour4,
        serde_json::from_str(include_str!("../BTC-USDT-SWAP-4h.json")).unwrap(),
        0.01,
        0.0,
    );

    let config = Config::new()
        .initial_margin(1000.0)
        .quantity(Unit::Quantity(0.01))
        .margin(Unit::Quantity(10.0))
        .lever(100)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004);

    let backtester = Backtester::new(exchange, config);

    let result = backtester
        .start(
            |cx| {
                if cx.position().is_none() {
                    if cci(cx.close, 20) <= -350.0 {
                        let result = cx.order(Side::BuyLong, 0.0);
                        println!(
                            "开仓委托结果 {} {} {:?}",
                            time_to_string(cx.time),
                            cx.close,
                            result
                        );
                    }
                } else {
                    if cci(cx.close, 20) >= 100.0 {
                        let result = cx.order(Side::BuySell, 0.0);
                        println!(
                            "平仓委托结果 {} {} {:?}",
                            time_to_string(cx.time),
                            cx.close,
                            result
                        );
                    }
                }
            },
            "BTC-USDT-SWAP",
            Level::Hour4,
            0,
        )
        .await
        .unwrap();

    println!("历史仓位 {:#?}", result);
    println!("所有盈亏 {}", result.iter().map(|v| v.profit).sum::<f64>());
}

#[tokio::test]
async fn test_my() {
    let k = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-1m.json")).unwrap();

    let exchange = LocalExchange::new()
        .push("BTC-USDT-SWAP", Level::Minute1, k.clone(), 0.01, 0.0)
        .push(
            "BTC-USDT-SWAP",
            Level::Minute15,
            k_convert(&k, Level::Minute15),
            0.01,
            0.0,
        );

    let config = Config::new()
        .initial_margin(1000.0)
        .quantity(Unit::Quantity(0.01))
        .margin(Unit::Quantity(10.0))
        .lever(100)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004);

    let backtester = Backtester::new(exchange, config);

    let mut ema_cache = EMACache::new();
    let mut macd_cache = MACDCache::new();
    let mut rsi_cache = RSICache::new();
    let mut last_macd = f64::NAN;

    let result = backtester
        .start_amplifier(
            |cx| {
                // close > ema 200
                // macd short > long
                // rsi14 >= 50
                let ema200 = ema_cache.ema(cx.close, 200);
                let (a, b, ..) = macd_cache.macd(cx.close, 12, 26, 9);
                let rsi14 = rsi_cache.rsi(cx.close, 14);

                if cx.close > ema200 {
                    if a > b && last_macd <= b {
                        if rsi14 > 50.0 && cx.position().is_none() {
                            let low = lowest(cx.low, 7);
                            let sp = cx.close + (cx.close - low) * 2.0;
                            let result = cx.order_profit_loss(
                                Side::BuyLong,
                                0.0,
                                Unit::Quantity(sp),
                                Unit::Quantity(low),
                            );
                            println!(
                                "做多 {} {} {} 止盈 {} 止损 {} {:?}",
                                cx.time,
                                time_to_string(cx.time),
                                cx.close,
                                sp,
                                low,
                                result
                            );
                        }
                    }
                }

                if cx.close < ema200 {
                    if a < b && last_macd >= b {
                        if rsi14 < 50.0 && cx.position().is_none() {
                            let high = highest(cx.high, 7);
                            let sp = cx.close - (high - cx.close) * 2.0;
                            let result = cx.order_profit_loss(
                                Side::SellShort,
                                0.0,
                                Unit::Quantity(sp),
                                Unit::Quantity(high),
                            );
                            println!(
                                "做空 {} {} {} 止盈 {} 止损 {} {:?}",
                                cx.time,
                                time_to_string(cx.time),
                                cx.close,
                                sp,
                                high,
                                result
                            );
                        }
                    }
                }

                last_macd = a;
            },
            "BTC-USDT-SWAP",
            Level::Minute1,
            Level::Minute15,
            1688384956000..,
        )
        .await
        .unwrap();

    // let k = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-4h.json")).unwrap();
    std::fs::write("./index.html", to_html(&k, &result)).unwrap();
    println!("所有盈亏 {}", result.iter().map(|v| v.profit).sum::<f64>());
    let mut q = 0.0;
    let mut w = 0.0;
    let mut a = 0;
    let mut b = 0;
    result.iter().for_each(|v| {
        if v.profit > 0.0 {
            a += 1;
            q += v.profit;
        } else if v.profit < 0.0 {
            b += 1;
            w += v.profit.abs();
        }
    });
    println!("盈亏比 {}", q / w);
    println!("胜率 {}", a as f64 / b as f64);
}

use auto_trading::*;

#[tokio::test]
async fn test_1() {
    let k = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-1m.json")).unwrap();

    let exchange = LocalExchange::new()
        .push("BTC-USDT-SWAP", Level::Minute1, k.clone(), 0.01, 0.0)
        .push(
            "BTC-USDT-SWAP",
            Level::Week1,
            k_convert(k, Level::Week1),
            0.01,
            0.0,
        );

    let config = Config::new()
        .initial_margin(1000.0)
        .lever(100)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004);

    let backtester = Backtester::new(exchange, config);

    let strategy = |cx: &mut Context| {
        println!(
            "{} {} {} {} {} {}",
            cx.time,
            time_to_string(cx.time),
            cx.open,
            cx.high,
            cx.low,
            cx.close,
        );
    };

    let result = backtester
        .start_amplifier(strategy, "BTC-USDT-SWAP", Level::Minute1, Level::Week1, 0)
        .await;

    println!("{:#?}", result);
}

#[tokio::test]
async fn test_2() {
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

    let config = Config::new()
        .initial_margin(1000.0)
        .lever(100)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004);

    let backtester = Backtester::new(exchange, config);

    let strategy = |cx: &mut Context| {
        println!(
            "{} {} {} {} {} {}",
            cx.time,
            time_to_string(cx.time),
            cx.open,
            cx.high,
            cx.low,
            cx.close,
        );
    };

    let result = backtester
        .start_amplifier(strategy, "BTC-USDT-SWAP", Level::Minute1, Level::Hour4, 0)
        .await;

    println!("{:#?}", result);
}

#[tokio::test]
async fn test_3() {
    let k = include_str!("../BTC-USDT-SWAP-4h.json");

    let exchange = LocalExchange::new().push(
        "BTC-USDT-SWAP",
        Level::Hour4,
        serde_json::from_str::<Vec<K>>(k).unwrap(),
        0.01,
        0.0,
    );

    let config = Config::new()
        .initial_margin(1000.0)
        .lever(100)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004);

    let backtester = Backtester::new(exchange, config);

    let strategy = |cx: &mut Context| {
        println!(
            "{} {} {} {} {} {}",
            cx.time,
            time_to_string(cx.time),
            cx.open,
            cx.high,
            cx.low,
            cx.close,
        );
    };

    let result = backtester
        .start_amplifier(strategy, "BTC-USDT-SWAP", Level::Hour4, Level::Hour4, 0)
        .await;

    println!("{:#?}", result);
}

#[tokio::test]
async fn test_4() {
    let k = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-4h.json")).unwrap();

    let exchange = LocalExchange::new()
        .push("BTC-USDT-SWAP", Level::Hour4, k.clone(), 0.01, 0.0)
        .push(
            "BTC-USDT-SWAP",
            Level::Hour12,
            k_convert(k, Level::Hour12),
            0.01,
            0.0,
        );

    let config = Config::new()
        .initial_margin(1000.0)
        .lever(100)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004);

    let backtester = Backtester::new(exchange, config);

    let strategy = |cx: &mut Context| {
        println!(
            "{} {} {} {} {} {}",
            cx.time,
            time_to_string(cx.time),
            cx.open,
            cx.high,
            cx.low,
            cx.close,
        );
    };

    let result = backtester
        .start_amplifier(strategy, "BTC-USDT-SWAP", Level::Hour4, Level::Hour12, 0)
        .await;

    println!("{:#?}", result);
}

#[tokio::test]
async fn test_5() {
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

    let config = Config::new()
        .initial_margin(1000.0)
        .lever(100)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004);

    let backtester = Backtester::new(exchange, config);

    let strategy = |cx: &mut Context| {
        if cx.time == 1686916800000 {
            let result = cx.order(Side::BuyLong, 25300.0);
            println!("{:?}", result);
        }

        if cx.time == 1686931200000 {
            println!("{:#?}", cx.delegate(1));
            assert!(cx.position().is_none(), "{:#?}", cx.position());
            std::process::exit(0);
        }
    };

    let result = backtester
        .start_amplifier(strategy, "BTC-USDT-SWAP", Level::Minute1, Level::Hour4, 0)
        .await;

    println!("{:#?}", result);
}

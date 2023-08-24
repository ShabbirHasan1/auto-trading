use auto_trading::*;

#[tokio::main]
async fn main() {
    let mut a = true;
    let mut b = true;

    let strategy = |cx: &mut Context| {
        if cci(cx.close, 20) <= -100.0 && a {
            let result = cx.order(Side::BuyLong, 0.0);
            println!("做多 {} {} {:?}", time_to_string(cx.time), cx.time, result);
            // let result = cx.order(Side::BuyLong, cx.close - 100);
            // println!("做多 {} {} {:?}", time_to_string(cx.time), cx.time, result);
            // let result = cx.order(Side::BuyLong, cx.close - 200);
            // println!("做多 {} {} {:?}", time_to_string(cx.time), cx.time, result);

            let result = cx.order(Side::SellShort, 0.0);
            println!("做空 {} {} {:?}", time_to_string(cx.time), cx.time, result);
            // let result = cx.order(Side::SellShort, cx.close + 100);
            // println!("做空 {} {} {:?}", time_to_string(cx.time), cx.time, result);
            // let result = cx.order(Side::SellShort, cx.close + 200);
            // println!("做空 {} {} {:?}", time_to_string(cx.time), cx.time, result);
            a = false;
        }
    };

    let mut bourse = LocalExchange::new();

    bourse
        .level_k(
            "BTC-USDT-SWAP",
            Level::Minute1,
            serde_json::from_str::<Vec<K>>(
                &std::fs::read_to_string("./BTC-USDT-SWAP-1m.txt").unwrap(),
            )
            .unwrap(),
        )
        .min_unit("BTC-USDT-SWAP", 0.01)
        .level_k(
            "BTC-USDT-SWAP",
            Level::Hour4,
            serde_json::from_str::<Vec<K>>(
                &std::fs::read_to_string("./BTC-USDT-SWAP-4h.txt").unwrap(),
            )
            .unwrap(),
        )
        .min_unit("BTC-USDT-SWAP", 0.01);

    let config = Config::new()
        .initial_margin(1000.0)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004)
        .lever(100)
        .margin(Unit::Proportion(2.0));

    let bt = Backtester::new(bourse, config);

    // 1659539044000..1691075044000
    let result = bt
        .start(strategy, "BTC-USDT-SWAP", Level::Hour4, 1687377600000..)
        .await
        .unwrap();

    println!("{:#?}", result);
    // std::fs::write("./list.txt", format!("{:#?}", result));

    println!("{}", result.iter().map(|v| v.profit).sum::<f64>());
}

#[tokio::test]
async fn get_k() {
    let okx = Okx::new().unwrap();

    let mut result = Vec::new();

    let mut end = 0;
    loop {
        println!("{}", end);
        let v = okx.get_k("BTC-USDT-SWAP", Level::Minute1, end).await;

        if v.is_err() {
            continue;
        }

        let v = v.unwrap();

        if let Some(k) = v.last() {
            end = k.time;
            result.extend(v);
            // tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        } else {
            break;
        }
    }

    let c = "[".to_string()
        + &result
            .iter()
            .map(|v| serde_json::to_string(v).unwrap())
            .collect::<Vec<String>>()
            .join(",")
        + "]";

    std::fs::write("./BTC-USDT-SWAP-1m.txt", c).unwrap();
}

use auto_trading::*;

#[tokio::main]
async fn main() {
    let mut flag = true;
    let mut acci = 0.0;

    let strategy = |cx: &mut Context| {
        let a = cci(&cx.close[1..], 20);
        let b = cci(cx.close, 20);

        if acci == 0.0 && a <= -100.0 && a <= b {
            acci = sma_map(cx.close, 5, |v| cci(v, 20));
        }

        if acci != 0.0 && cci(cx.close, 20) >= acci && flag {
            let result = cx.order(Side::BuyLong, 0.0);
            println!(
                "做多 {} {} {} {:?}",
                cx.time,
                time_to_string(cx.time),
                cx.close[0],
                result
            );

            flag = false;
        }

        if cci(cx.close, 20) >= 100.0 && !flag {
            let result = cx.order(Side::BuySell, 0.0);
            println!(
                "平多 {} {} {} {:?}",
                cx.time,
                time_to_string(cx.time),
                cx.close[0],
                result
            );
            flag = true;
            acci = 0.0;
        }
    };

    let mut bourse = LocalBourse::new();

    bourse
        .level_k(
            "BTC-USDT-SWAP",
            Level::Hour4,
            serde_json::from_str::<Vec<K>>(
                &std::fs::read_to_string("./BTC-USDT-SWAP-4H.txt").unwrap(),
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
        .margin(Unit::Proportion(10.0))
        .isolated(true);

    let bt = Backtester::new(bourse, config);

    // 1659539044000..1691075044000
    let result = bt
        .start(strategy, "BTC-USDT-SWAP", Level::Hour4, 0)
        .await
        .unwrap();

    // println!("{:#?}", result);
    // std::fs::write("./list.txt", format!("{:#?}", result));

    println!("{}", result.iter().map(|v| v.profit).sum::<f64>());
}

#[tokio::test]
async fn get_k() {
    let okx = Okx::new().unwrap();

    let mut result = Vec::new();

    let mut end = 0;
    loop {
        let v = okx.get_k("BTC-USDT-SWAP", Level::Hour4, end).await.unwrap();

        if let Some(k) = v.last() {
            end = k.time;
            result.extend(v);
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
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

    std::fs::write("./BTC-USDT-SWAP-4H.txt", c).unwrap();
}

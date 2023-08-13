use auto_trading::*;

#[tokio::main]
async fn main() {
    let mut ok = true;

    let strategy = |cx: &mut Context| {
        let cci20 = cci(cx.close, 20);

        if cci20 <= -100.0 && ok {
            let result = cx.order(Side::BuyLong, 0.0);
            println!(
                "做多 {} {} {:?}",
                time_to_string(cx.time),
                cx.close[0],
                result
            );
            ok = false;
        }

        if cci20 >= 90.0 && !ok {
            let result = cx.order(Side::BuySell, 0.0);
            println!(
                "平多 {} {} {:?}",
                time_to_string(cx.time),
                cx.close[0],
                result
            );
            ok = true;
        }
    };

    let mut bourse = LocalBourse::new();

    bourse
        .level_k(
            "ETH-USDT-SWAP",
            Level::Hour4,
            serde_json::from_str::<Vec<K>>(&std::fs::read_to_string("./4hk.txt").unwrap()).unwrap(),
        )
        .min_unit("ETH-USDT-SWAP", 0.1);

    let config = Config::new()
        .initial_margin(1000.0)
        .lever(35)
        .margin(10)
        .isolated(true);

    let bt = Backtester::new(bourse, config);

    // 1659539044000..1691075044000
    let result = bt
        .start(
            strategy,
            "ETH-USDT-SWAP",
            Level::Hour4,
            Level::Minute1,
            1683555251000..1691075044000,
        )
        .await
        .unwrap();
}

// let okx = Okx::new().unwrap();

// let mut result = Vec::new();

// let mut end = 0;
// loop {
//     let v = okx.get_k("ETH-USDT-SWAP", Level::Hour4, end).await.unwrap();

//     if let Some(k) = v.last() {
//         end = k.time;
//         result.extend(v);
//         tokio::time::sleep(std::time::Duration::from_millis(100)).await;
//     } else {
//         break;
//     }
// }

// // let qwe = okx.get_k("ETH-USDT-SWAP", Level::Minute1, 0).await.unwrap();

// println!("{}", result.len());

// let c = result
//     .iter()
//     .map(|v| v.to_string())
//     .collect::<Vec<String>>()
//     .join(",");

// std::fs::write("./k.txt", c).unwrap();

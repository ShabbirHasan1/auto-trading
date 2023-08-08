use auto_trading::*;

#[tokio::main]
async fn main() {
    let mut ok = true;

    // TODO: 实现 close 到 Unit

    let strategy = |cx: &mut Context| {
        // 跌破前低做多
        let low = lowest(&cx.close[1..], 30);
        let highest = highest(&cx.close[1..], 30);
        if cx.close < low {
            if ok {
                println!(
                    "{}: 做多 {}: {:?}",
                    time_to_string(cx.time),
                    cx.close[0],
                    cx.order(Side::BuyLong, 0.0)
                );
                ok = false;
            }
        } else if cx.close < highest && !ok {
            println!(
                "{}: 平多 {}: {:?}",
                time_to_string(cx.time),
                cx.close[0],
                cx.order(Side::BuySell, 0.0)
            );
            ok = true;
        }
    };

    let mut bourse = LocalBourse::new();

    bourse
        .level_k(
            "ETH-USDT-SWAP",
            Level::Hour4,
            serde_json::from_str::<Vec<K>>(&std::fs::read_to_string("./k.txt").unwrap()).unwrap(),
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
            1659539044000..1691075044000,
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

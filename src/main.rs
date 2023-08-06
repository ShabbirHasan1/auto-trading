use auto_trading::*;

#[tokio::main]
async fn main() {
    let mut x = 0;

    let strategy = |cx: &mut Context| {
        // 跌破前低做多
        let low = lowest(&cx.close[1..], 30);
        if cx.close < low {
            if x <= 3 {
                println!("做多 {}, time {}", cx.close[0], cx.time);
                cx.order(Side::BuyLong, 0.0).unwrap();
                x += 1;
            }
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
        .lever(20)
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

// return;

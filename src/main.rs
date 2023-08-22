use auto_trading::*;

#[tokio::main]
async fn main() {
    let mut start = 2000.0;
    for i in 0..180 {
        let lx = start * 0.09;
        start += lx;
        println!(
            "第 {} 天 利息 {} 换算人民币 {} 余额 {} 换算人民币 {}",
            i + 1,
            lx,
            lx * 0.075 * 7.2,
            start,
            start * 0.075 * 7.2
        );
    }

    // let mut flag = true;
    // let mut count = 0;
    // let strategy = |cx: &mut Context| {
    //     if cx.time >= 1687377600000 && cx.low <= 29800 && flag {
    //         let result = cx.order_condition(Side::BuyLong, 29862.0, 0, 32000, 29200, 0, 0);
    //         println!("做多 {} {} {:?}", time_to_string(cx.time), cx.time, result);
    //         count += 1;

    //         if count == 3 {
    //             flag = false;
    //         }
    //     }
    // };

    // let mut bourse = LocalBourse::new();

    // bourse
    //     .level_k(
    //         "BTC-USDT-SWAP",
    //         Level::Minute1,
    //         serde_json::from_str::<Vec<K>>(
    //             &std::fs::read_to_string("./BTC-USDT-SWAP-1m.txt").unwrap(),
    //         )
    //         .unwrap(),
    //     )
    //     .min_unit("BTC-USDT-SWAP", 0.01)
    //     .level_k(
    //         "BTC-USDT-SWAP",
    //         Level::Hour4,
    //         serde_json::from_str::<Vec<K>>(
    //             &std::fs::read_to_string("./BTC-USDT-SWAP-4h.txt").unwrap(),
    //         )
    //         .unwrap(),
    //     )
    //     .min_unit("BTC-USDT-SWAP", 0.01);

    // let config = Config::new()
    //     .initial_margin(1000.0)
    //     .open_fee(0.0002)
    //     .close_fee(0.0005)
    //     .maintenance(0.004)
    //     .lever(100)
    //     .margin(Unit::Proportion(2.0))
    //     .isolated(true);

    // let bt = Backtester::new(bourse, config);

    // // 1659539044000..1691075044000
    // let result = bt
    //     .start(strategy, "BTC-USDT-SWAP", Level::Hour4, 1687665600000..)
    //     .await
    //     .unwrap();

    // println!("{:#?}", result);
    // // std::fs::write("./list.txt", format!("{:#?}", result));

    // println!("{}", result.iter().map(|v| v.profit).sum::<f64>());
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

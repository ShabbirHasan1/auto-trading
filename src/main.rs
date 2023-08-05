use auto_trading::*;

#[tokio::main]
async fn main() {
    let strategy = |cx: &mut Context| {
        println!("{}", time_to_string(cx.time));
    };

    let okx = Okx::new().unwrap();
    let config = Config::new().initial_margin(1000.0).lever(1).margin(1);
    let bt = Backtester::new(okx, config);

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

    println!("{:#?}", result);
}

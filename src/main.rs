use auto_trading::*;

#[tokio::main]
async fn main() {
    let strategy = |cx: &Context| {
        println!("{:?}", cx.close[0]);
    };
    let okx = Okx::new().unwrap().base_url("https://www.rkdfs.com");
    let config = Config::new().initial_margin(1000.0).lever(100).margin(10);
    let bt = Backtester::new(okx, config);
    let result = bt
        .start(
            strategy,
            "ETH-USDT",
            Level::Hour4,
            Level::Minute1,
            1659539044000..1691075044000,
        )
        .await
        .unwrap();
    println!("{:#?}", result);
}

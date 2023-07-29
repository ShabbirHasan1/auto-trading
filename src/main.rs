use auto_trading::*;

#[tokio::main]
async fn main() {
    let okx = Okx::new().unwrap();
    let config = Config::new().initial_margin(1000.0).lever(100).margin(10);
    let bt = Backtester::new(okx).config(config);
    let result = bt.start("ETH-USDT-SWAP", Level::Hour4, 0).await.unwrap();
    println!("{:#?}", result);
}

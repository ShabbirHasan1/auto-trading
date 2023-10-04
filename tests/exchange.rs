use auto_trading::*;

#[tokio::test]
async fn okx_get_k() {
    let exchange = Okx::new().unwrap();

    let k1 = exchange
        .get_k("BTC-USDT-SWAP", Level::Hour1, 0)
        .await
        .unwrap();

    let k2 = exchange
        .get_k("BTC-USDT", Level::Hour1, k1.last().unwrap().time)
        .await
        .unwrap();

    println!("{}", k1[0].open);
    println!("{}", k2[0].open);
    println!("{}", time_to_string(k1[0].time));
    println!("{}", time_to_string(k1.last().unwrap().time));
    println!("{}", time_to_string(k2[0].time));
    println!("{}", time_to_string(k2.last().unwrap().time));

    assert!(k1.last().unwrap().time != k2[0].time);
}

#[tokio::test]
async fn okx_get_min_size() {
    let exchange = Okx::new().unwrap();
    let x = exchange.get_min_size("BTC-USDT-SWAP").await.unwrap();
    assert!(x == 0.01);
    let x = exchange.get_min_size("BTC-USDT").await.unwrap();
    assert!(x == 0.00001);
}

#[tokio::test]
async fn binance_get_k() {
    let exchange = Binance::new().unwrap();

    let k1 = exchange
        .get_k("BTC-USDT-SWAP", Level::Hour1, 0)
        .await
        .unwrap();

    let k2 = exchange
        .get_k("BTC-USDT", Level::Hour1, k1.last().unwrap().time)
        .await
        .unwrap();

    println!("{}", k1[0].open);
    println!("{}", k2[0].open);
    println!("{}", time_to_string(k1[0].time));
    println!("{}", time_to_string(k1.last().unwrap().time));
    println!("{}", time_to_string(k2[0].time));
    println!("{}", time_to_string(k2.last().unwrap().time));

    assert!(k1.last().unwrap().time != k2[0].time);
}

#[tokio::test]
async fn binance_get_min_size() {
    let exchange = Binance::new().unwrap();
    let x = exchange.get_min_size("BTC-USDT-SWAP").await.unwrap();
    assert!(x == 0.001);
    let x = exchange.get_min_size("BTC-USDT").await.unwrap();
    assert!(x == 0.00001);
}

#[tokio::test]
async fn binance_get_min_notional() {
    let exchange = Binance::new().unwrap();
    let x = exchange.get_min_notional("BTC-USDT-SWAP").await.unwrap();
    assert!(x == 5.0);
    let x = exchange.get_min_notional("BTC-USDT").await.unwrap();
    assert!(x == 5.0);
}

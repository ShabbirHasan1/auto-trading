# 这是什么？

一个通用的回测框架，支持欧易，币安平台。

# 依赖本库

```rust
auto-trading = "0.3.0"
```
或者 

```rust
cargo add auto-trading
```

# 例子1

```rust
use auto_trading::*;

#[tokio::test]
async fn test1() {
    let mut flag = true;

    let strategy = |cx: &mut Context| {
        if cci(cx.close, 20) <= -100.0 {
            let result = cx.order(Side::BuyLong, 0.0);
            println!("做多 {} {} {:?}", time_to_string(cx.time), cx.close, result);
            flag = false;
        }

        if cci(cx.close, 20) >= 100.0 {
            let result = cx.order(Side::BuySell, 0.0);
            println!("平多 {} {} {:?}", time_to_string(cx.time), cx.close, result);
            flag = true;
        }
    };

    let exchange = Okx::new().unwrap();

    let config = Config::new()
        .initial_margin(1000.0)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004)
        .lever(100)
        // 保证金为策略开仓价值的 3 倍，策略默认开仓价值为 1 张
        .margin(Unit::Proportion(3.0));

    let bt = Backtester::new(exchange, config);

    // 时间范围
    // 0 和 .. 表示全部
    // a.. 和 ..b 和 a..b 表示范围
    let result = bt
        .start(
            strategy,
            "BTC-USDT-SWAP",
            Level::Hour4,
            1659539044000..1691075044000,
        )
        .await
        .unwrap();

    println!("历史仓位\n{:#?}", result);

    println!("盈亏 {}", result.iter().map(|v| v.profit).sum::<f64>());
}
```

# 例子2

```rust
use auto_trading::*;

#[tokio::test]
async fn test2() {
    let strategy = |cx: &mut Context| {
        if cx.position("").is_none() && cci(cx.close, 20) <= -100.0 {
            let result =
                cx.order_condition(Side::BuyLong, 0.0, 0, cx.close + 300, cx.close - 200, 0, 0);
            println!("做多 {} {} {:?}", time_to_string(cx.time), cx.close, result);
        }
    };

    // 本地交易所
    let exchange = LocalExchange::new()
        .level_k(
            "BTC-USDT-SWAP",
            Level::Hour4,
            serde_json::from_slice::<Vec<K>>(&std::fs::read("BTC-USDT-SWAP.json").unwrap())
                .unwrap(),
        )
        .unit("BTC-USDT-SWAP", 0.01);

    let config = Config::new()
        .initial_margin(1000.0)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004)
        .lever(100)
        // 保证金为策略开仓价值的 3 倍，策略默认开仓价值为 1 张
        .margin(Unit::Proportion(3.0));

    let bt = Backtester::new(exchange, config);

    // 时间范围
    // 0 和 .. 表示全部
    // a.. 和 ..b 和 a..b 表示范围
    let result = bt
        .start(
            strategy,
            "BTC-USDT-SWAP",
            Level::Hour4,
            1659539044000..1691075044000,
        )
        .await
        .unwrap();

    println!("历史仓位\n{:#?}", result);

    println!("盈亏 {}", result.iter().map(|v| v.profit).sum::<f64>());
}
```

use auto_trading::*;

#[test]
fn test_config1() {}

#[test]
fn test_config2() {
    // 测试数量
    let config = Config::new()
        .initial_margin(1000.0)
        .quantity(Unit::Quantity(0.01))
        .margin(Unit::Quantity(200.0));
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 114514,
            open: 10000.0,
            high: 25000.0,
            low: 5000.0,
            close: 20000.0,
        },
    );
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        0.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    assert!(
        me.delegate(1).unwrap()
            == DelegateState::Single(Delegate {
                side: Side::BuyLong,
                price: Price::GreaterThanMarket(20000.0),
                quantity: 0.01,
                margin: 200.0,
                append_margin: 0.0
            }),
        "{:#?}",
        me
    );
}

#[test]
fn test_config3() {
    // 测试比例
    let config = Config::new()
        .initial_margin(1000.0)
        .quantity(Unit::Proportion(0.3))
        .margin(Unit::Proportion(0.6));
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 114514,
            open: 10000.0,
            high: 25000.0,
            low: 5000.0,
            close: 20000.0,
        },
    );
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        0.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    assert!(
        me.delegate(1).unwrap()
            == DelegateState::Single(Delegate {
                side: Side::BuyLong,
                price: Price::GreaterThanMarket(20000.0),
                quantity: 0.01,
                margin: 600.0,
                append_margin: 0.0
            }),
        "{:#?}",
        me
    );
}

#[test]
fn test_config4() {
    // 测试默认
    let config = Config::new().initial_margin(1000.0);
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 114514,
            open: 1000.0,
            high: 2500.0,
            low: 500.0,
            close: 2000.0,
        },
    );
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        0.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    assert!(
        me.delegate(1).unwrap()
            == DelegateState::Single(Delegate {
                side: Side::BuyLong,
                price: Price::GreaterThanMarket(2000.0),
                quantity: 0.01,
                margin: 20.0,
                append_margin: 0.0
            }),
        "{:#?}",
        me
    );
}

#[test]
fn test_order1() {
    // 测试止盈止损
    let config = Config::new().initial_margin(1000.0);
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 114514,
            open: 1000.0,
            high: 2500.0,
            low: 500.0,
            close: 2000.0,
        },
    );
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        0.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Quantity(2100.0),
        Unit::Quantity(1950.0),
        Unit::Quantity(3000.0),
        Unit::Quantity(1000.0),
    )
    .unwrap();
    assert!(
        me.delegate(1).unwrap()
            == DelegateState::OpenProfitLoss(
                Delegate {
                    side: Side::BuyLong,
                    price: Price::GreaterThanMarket(2000.0),
                    quantity: 0.01,
                    margin: 20.0,
                    append_margin: 0.0
                },
                Delegate {
                    side: Side::BuySell,
                    price: Price::GreaterThanLimit(2100.0, 3000.0),
                    quantity: 0.01,
                    margin: 20.0,
                    append_margin: 0.0
                },
                Delegate {
                    side: Side::BuySell,
                    price: Price::LessThanLimit(1950.0, 1000.0),
                    quantity: 0.01,
                    margin: 20.0,
                    append_margin: 0.0
                },
            ),
        "{:#?}",
        me
    );
}

#[test]
fn test_order2() {
    // 测试止盈止损百分比
    let config = Config::new().initial_margin(1000.0);
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 114514,
            open: 1000.0,
            high: 2500.0,
            low: 500.0,
            close: 2000.0,
        },
    );
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        0.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Proportion(0.5),
        Unit::Proportion(0.3),
        Unit::Proportion(0.7),
        Unit::Proportion(0.5),
    )
    .unwrap();
    assert!(
        me.delegate(1).unwrap()
            == DelegateState::OpenProfitLoss(
                Delegate {
                    side: Side::BuyLong,
                    price: Price::GreaterThanMarket(2000.0),
                    quantity: 0.01,
                    margin: 20.0,
                    append_margin: 0.0
                },
                Delegate {
                    side: Side::BuySell,
                    price: Price::GreaterThanLimit(2000.0 + 2000.0 * 0.5, 2000.0 + 2000.0 * 0.7),
                    quantity: 0.01,
                    margin: 20.0,
                    append_margin: 0.0
                },
                Delegate {
                    side: Side::BuySell,
                    price: Price::LessThanLimit(2000.0 - 2000.0 * 0.3, 2000.0 - 2000.0 * 0.5),
                    quantity: 0.01,
                    margin: 20.0,
                    append_margin: 0.0
                },
            ),
        "{:#?}",
        me
    );
}

#[test]
fn test_order3() {
    // 测试开多，然后平空
    let config = Config::new().initial_margin(1000.0);
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 114514,
            open: 1000.0,
            high: 2500.0,
            low: 500.0,
            close: 2000.0,
        },
    );
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        0.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.update();
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 114514,
            open: 1000.0,
            high: 2500.0,
            low: 500.0,
            close: 2000.0,
        },
    );
    let result = me.order(
        "BTC-USDT-SWAP",
        Side::SellLong,
        0.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    );
    println!("{}", result.unwrap_err());
}

#[test]
fn test_order4() {
    // 测试开空，然后平多
    let config = Config::new().initial_margin(1000.0);
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 114514,
            open: 1000.0,
            high: 2500.0,
            low: 500.0,
            close: 2000.0,
        },
    );
    me.order(
        "BTC-USDT-SWAP",
        Side::SellShort,
        0.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.update();
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 114514,
            open: 1000.0,
            high: 2500.0,
            low: 500.0,
            close: 2000.0,
        },
    );
    let result = me.order(
        "BTC-USDT-SWAP",
        Side::BuySell,
        0.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    );
    println!("{}", result.unwrap_err());
}

#[test]
fn test_order_args() {
    // 测试委托的参数合法性
    let config = Config::new().initial_margin(1000.0);
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 114514,
            open: 1000.0,
            high: 2500.0,
            low: 500.0,
            close: 2000.0,
        },
    );
    let result = me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        0.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Quantity(1950.0),
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    );
    println!("{}", result.unwrap_err());
    let result = me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        0.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Quantity(2100.0),
        Unit::Ignore,
        Unit::Ignore,
    );
    println!("{}", result.unwrap_err());
    let result = me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        2500.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Quantity(2100.0),
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    );

    println!("{}", result.unwrap_err());
    let result = me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        2500.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Quantity(3000.0),
        Unit::Ignore,
        Unit::Ignore,
    );
    println!("{}", result.unwrap_err());
}

#[test]
fn test_update1() {
    // 测试强平价格是否准确，还有强平是否有效
    let config = Config::new()
        .initial_margin(1000.0)
        .lever(100)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004);
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 114514,
            open: 10000.0,
            high: 25000.0,
            low: 5000.0,
            close: 20000.0,
        },
    );
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        0.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Quantity(21000.0),
        Unit::Quantity(19500.0),
        Unit::Quantity(30000.0),
        Unit::Quantity(10000.0),
    )
    .unwrap();
    me.update();
    assert!(
        me.position("BTC-USDT-SWAP").unwrap().liquidation_price == 19880.1,
        "{:#?}",
        me
    );
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 1919810,
            open: 10000.0,
            high: 25000.0,
            low: 19800.0,
            close: 20000.0,
        },
    );
    me.update();
    assert!(me.position("BTC-USDT-SWAP").is_none(), "{:#?}", me);
}

#[test]
fn test_update2() {
    // 测试开仓均价是否准确
    let config = Config::new()
        .initial_margin(10000.0)
        .margin(Unit::Quantity(1000.0))
        .lever(100)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004);
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 114514,
            open: 10000.0,
            high: 25000.0,
            low: 5000.0,
            close: 20000.0,
        },
    );
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        20000.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.update();
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 114514,
            open: 10000.0,
            high: 25000.0,
            low: 19500.0,
            close: 19600.0,
        },
    );
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        19800.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.update();
    assert!(
        me.position("BTC-USDT-SWAP").unwrap().open_price == 19900.0,
        "{:#?}",
        me
    );
}

#[test]
fn test_update3() {
    // 测试止盈是否有效
    let config = Config::new()
        .initial_margin(1000.0)
        .margin(Unit::Quantity(100.0))
        .lever(10)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004);
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 1,
            open: 10000.0,
            high: 25000.0,
            low: 5000.0,
            close: 20000.0,
        },
    );
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        0.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Quantity(20100.0),
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.update();
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 2,
            open: 20000.0,
            high: 25000.0,
            low: 15000.0,
            close: 20050.0,
        },
    );
    me.update();
    assert!(me.history()[0].close_price == 20100.0, "{:#?}", me);
}

#[test]
fn test_update4() {
    // 测试止损是否有效
    let config = Config::new()
        .initial_margin(1000.0)
        .margin(Unit::Quantity(100.0))
        .lever(10)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004);
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 1,
            open: 10000.0,
            high: 25000.0,
            low: 5000.0,
            close: 20000.0,
        },
    );
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        20000.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Quantity(19900.0),
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.update();
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 2,
            open: 20000.0,
            high: 25000.0,
            low: 15000.0,
            close: 19950.0,
        },
    );
    me.update();
    assert!(me.history()[0].close_price == 19900.0, "{:#?}", me);
}

#[test]
fn test_update5() {
    // 测试止盈触发价
    let config = Config::new()
        .initial_margin(1000.0)
        .margin(Unit::Quantity(100.0))
        .lever(10)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004);
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 1,
            open: 10000.0,
            high: 25000.0,
            low: 5000.0,
            close: 20000.0,
        },
    );
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        20000.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Quantity(20100.0),
        Unit::Ignore,
        Unit::Quantity(20200.0),
        Unit::Ignore,
    )
    .unwrap();
    me.update();
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 2,
            open: 20000.0,
            high: 20120.0,
            low: 15000.0,
            close: 20050.0,
        },
    );
    me.update();
    assert!(
        me.delegate(1).unwrap()
            == DelegateState::Single(Delegate {
                side: Side::BuySell,
                price: Price::GreaterThanMarket(20200.0),
                quantity: 0.01,
                margin: 100.0,
                append_margin: 0.0
            }),
        "{:#?}",
        me
    );
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 3,
            open: 20000.0,
            high: 25000.0,
            low: 15000.0,
            close: 20050.0,
        },
    );
    me.update();
    assert!(me.history()[0].close_price == 20200.0, "{:#?}", me);
}

#[test]
fn test_update6() {
    // 测试止损触发价
    let config = Config::new()
        .initial_margin(1000.0)
        .margin(Unit::Quantity(100.0))
        .lever(10)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004);
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 1,
            open: 10000.0,
            high: 25000.0,
            low: 5000.0,
            close: 20000.0,
        },
    );
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        20000.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Quantity(19500.0),
        Unit::Ignore,
        Unit::Quantity(19000.0),
    )
    .unwrap();
    me.update();
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 2,
            open: 20000.0,
            high: 25000.0,
            low: 19400.0,
            close: 19450.0,
        },
    );
    me.update();
    assert!(
        me.delegate(1).unwrap()
            == DelegateState::Single(Delegate {
                side: Side::BuySell,
                price: Price::LessThanMarket(19000.0),
                quantity: 0.01,
                margin: 100.0,
                append_margin: 0.0
            }),
        "{:#?}",
        me
    );
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 2,
            open: 20000.0,
            high: 25000.0,
            low: 15000.0,
            close: 19400.0,
        },
    );
    me.update();
    assert!(me.history()[0].close_price == 19000.0, "{:#?}", me);
}

#[test]
fn test_update7() {
    // 测试做多两次，然后逐个卖出
    let config = Config::new()
        .initial_margin(10000.0)
        .lever(100)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004);
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 1,
            open: 21000.0,
            high: 30000.0,
            low: 20000.0,
            close: 29000.0,
        },
    );
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        29100.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        30100.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.update();
    assert!(me.delegate(1).is_none(), "{:#?}", me);
    assert!(me.delegate(2).is_some(), "{:#?}", me);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 2,
            open: 21000.0,
            high: 30200.0,
            low: 29500.0,
            close: 29500.0,
        },
    );
    me.update();
    assert!(me.delegate(2).is_none(), "{:#?}", me);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 2,
            open: 21000.0,
            high: 30500.0,
            low: 29500.0,
            close: 29500.0,
        },
    );
    me.order(
        "BTC-USDT-SWAP",
        Side::BuySell,
        30400.0,
        Unit::Quantity(0.01),
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.update();
    assert!(me.position("BTC-USDT-SWAP").is_some(), "{:#?}", me);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 3,
            open: 40000.0,
            high: 70000.0,
            low: 29600.0,
            close: 29800.0,
        },
    );
    me.order(
        "BTC-USDT-SWAP",
        Side::BuySell,
        29700.0,
        Unit::Quantity(0.01),
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.update();
    assert!(me.history()[0].open_price == 29600.0, "{:#?}", me);
    assert!(me.history()[0].log[2].price == 30400.0, "{:#?}", me);
    assert!(me.history()[0].log[2].margin == 2.96, "{:#?}", me);
    assert!(me.history()[0].log[2].profit == 8.0, "{:#?}", me);
    assert!(me.history()[0].log[3].price == 29700.0, "{:#?}", me);
    assert!(me.history()[0].log[3].margin == 2.96, "{:#?}", me);
    assert!(me.history()[0].log[3].profit == 1.0, "{:#?}", me);
}

#[test]
fn test_update8() {
    // 测试对冲
    let config = Config::new()
        .initial_margin(10000.0)
        .lever(100)
        .open_fee(0.0002)
        .close_fee(0.0005)
        .maintenance(0.004);
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 1,
            open: 21000.0,
            high: 30000.0,
            low: 28900.0,
            close: 29000.0,
        },
    );
    // 做多 0.01 做空 0.01
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        29100.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.order(
        "BTC-USDT-SWAP",
        Side::SellShort,
        29200.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.update();
    assert!(me.history()[0].close_price == 29200.0, "{:#?}", me);
    // 做多 0.01 做空 0.02
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        29100.0,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.order(
        "BTC-USDT-SWAP",
        Side::SellShort,
        29200.0,
        Unit::Quantity(0.02),
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.update();
    assert!(
        me.history()[1].close_price == 29200.0
            && me.history()[1].quantity == 0.01
            && me.history()[1].margin == 2.91,
        "{:#?}",
        me
    );
    assert!(
        me.position("BTC-USDT-SWAP").unwrap().open_price == 29200.0
            && me.position("BTC-USDT-SWAP").unwrap().quantity == 0.01
            && me.position("BTC-USDT-SWAP").unwrap().margin == 2.92
    );
}

#[test]
fn test_update9() {
    // 测试对冲，追加保证金的情况
    let config = Config::new().initial_margin(10000.0).lever(100);
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 1,
            open: 21000.0,
            high: 30000.0,
            low: 28900.0,
            close: 29000.0,
        },
    );
    // 做多 0.01 做空 0.01
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        29100.0,
        Unit::Quantity(0.01),
        Unit::Quantity(3.0),
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.order(
        "BTC-USDT-SWAP",
        Side::SellShort,
        29200.0,
        Unit::Quantity(0.01),
        Unit::Quantity(7.0),
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.update();
    assert!(
        me.history()[0].close_price == 29200.0
            && me.history()[0].log[1].quantity == 0.01
            && me.history()[0].log[1].margin == 10.0
            && me.balance() == 10001.0,
        "{:#?}",
        me
    );
}

#[test]
fn test_update10() {
    // 测试对冲，追加保证金的情况
    let config = Config::new().initial_margin(10000.0).lever(100);
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 1,
            open: 21000.0,
            high: 30000.0,
            low: 28900.0,
            close: 29000.0,
        },
    );
    // 做多 0.01 做空 0.02
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        29100.0,
        Unit::Quantity(0.01),
        Unit::Quantity(3.0),
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.order(
        "BTC-USDT-SWAP",
        Side::SellShort,
        29200.0,
        Unit::Quantity(0.02),
        Unit::Quantity(9.0),
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.update();
    assert!(
        me.history()[0].close_price == 29200.0
            && me.history()[0].log[1].quantity == 0.01
            && me.history()[0].log[1].margin == 7.5
            && me.balance() == 9996.5,
        "{:#?}",
        me
    );
}

#[test]
fn test_update11() {
    // 测试对冲，追加保证金的情况
    let config = Config::new().initial_margin(10000.0).lever(100);
    let mut me = MatchEngine::new(config);
    me.insert_product("BTC-USDT-SWAP", 0.01, 0.0);
    me.ready(
        "BTC-USDT-SWAP",
        K {
            time: 1,
            open: 21000.0,
            high: 30000.0,
            low: 28900.0,
            close: 29000.0,
        },
    );
    // 做多 0.05 做空 0.02
    me.order(
        "BTC-USDT-SWAP",
        Side::BuyLong,
        29100.0,
        Unit::Quantity(0.05),
        Unit::Quantity(19.0),
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.order(
        "BTC-USDT-SWAP",
        Side::SellShort,
        29200.0,
        Unit::Quantity(0.02),
        Unit::Quantity(9.0),
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
        Unit::Ignore,
    )
    .unwrap();
    me.update();
    assert!(
        me.position("BTC-USDT-SWAP").unwrap().log[1].price == 29200.0
            && me.position("BTC-USDT-SWAP").unwrap().log[1].quantity == 0.02
            && me.position("BTC-USDT-SWAP").unwrap().log[1].margin == 0.02 / 0.05 * 19.0 + 9.0
            && me.balance() == 9990.6,
        "{:#?}",
        me
    );
}

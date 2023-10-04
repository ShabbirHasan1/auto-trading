use auto_trading::*;

#[test]
fn test_k_convert1() {
    let array = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-1m.json")).unwrap();

    let x = std::time::SystemTime::now();

    let result = k_convert(&array, Level::Minute1);

    let x = std::time::SystemTime::now().duration_since(x).unwrap();

    println!("{}", x.as_millis());

    assert!(result[1].time - 1000 * 60 == result[2].time);
}

#[test]
fn test_k_convert2() {
    let array = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-1m.json")).unwrap();

    let x = std::time::SystemTime::now();

    let result = k_convert(&array, Level::Minute3);

    let x = std::time::SystemTime::now().duration_since(x).unwrap();

    println!("{}", x.as_millis());

    assert!(result[1].time - 1000 * 60 * 3 == result[2].time);
}

#[test]
fn test_k_convert3() {
    let array = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-1m.json")).unwrap();

    let x = std::time::SystemTime::now();

    let result = k_convert(&array, Level::Minute5);

    let x = std::time::SystemTime::now().duration_since(x).unwrap();

    println!("{}", x.as_millis());

    assert!(result[1].time - 1000 * 60 * 5 == result[2].time);
}

#[test]
fn test_k_convert4() {
    let array = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-1m.json")).unwrap();

    let x = std::time::SystemTime::now();

    let result = k_convert(&array, Level::Minute15);

    let x = std::time::SystemTime::now().duration_since(x).unwrap();

    println!("{}", x.as_millis());

    assert!(result[1].time - 1000 * 60 * 15 == result[2].time);
}

#[test]
fn test_k_convert5() {
    let array = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-1m.json")).unwrap();

    let x = std::time::SystemTime::now();

    let result = k_convert(&array, Level::Minute30);

    let x = std::time::SystemTime::now().duration_since(x).unwrap();

    println!("{}", x.as_millis());

    assert!(result[1].time - 1000 * 60 * 30 == result[2].time);
}

#[test]
fn test_k_convert6() {
    let array = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-1m.json")).unwrap();

    let x = std::time::SystemTime::now();

    let result = k_convert(&array, Level::Hour1);

    let x = std::time::SystemTime::now().duration_since(x).unwrap();

    println!("{}", x.as_millis());

    assert!(result[1].time - 1000 * 60 * 60 == result[2].time);
}

#[test]
fn test_k_convert7() {
    let array = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-1m.json")).unwrap();

    let x = std::time::SystemTime::now();

    let result = k_convert(&array, Level::Hour2);

    let x = std::time::SystemTime::now().duration_since(x).unwrap();

    println!("{}", x.as_millis());

    assert!(result[1].time - 1000 * 60 * 60 * 2 == result[2].time);
}

#[test]
fn test_k_convert8() {
    let array = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-1m.json")).unwrap();

    let x = std::time::SystemTime::now();

    let result = k_convert(&array, Level::Hour4);

    let x = std::time::SystemTime::now().duration_since(x).unwrap();

    println!("{}", x.as_millis());

    assert!(result[1].time - 1000 * 60 * 60 * 4 == result[2].time);
}

#[test]
fn test_k_convert9() {
    let array = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-1m.json")).unwrap();

    let x = std::time::SystemTime::now();

    let result = k_convert(&array, Level::Hour6);

    let x = std::time::SystemTime::now().duration_since(x).unwrap();

    println!("{}", x.as_millis());

    assert!(result[1].time - 1000 * 60 * 60 * 6 == result[2].time);
}

#[test]
fn test_k_convert10() {
    let array = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-1m.json")).unwrap();

    let x = std::time::SystemTime::now();

    let result = k_convert(&array, Level::Hour12);

    let x = std::time::SystemTime::now().duration_since(x).unwrap();

    println!("{}", x.as_millis());

    assert!(result[1].time - 1000 * 60 * 60 * 12 == result[2].time);
}

#[test]
fn test_k_convert11() {
    let array = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-1m.json")).unwrap();

    let x = std::time::SystemTime::now();

    let result = k_convert(&array, Level::Day1);

    let x = std::time::SystemTime::now().duration_since(x).unwrap();

    println!("{}", x.as_millis());

    assert!(result[1].time - 1000 * 60 * 60 * 24 == result[2].time);
}

#[test]
fn test_k_convert12() {
    let array = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-1m.json")).unwrap();

    let x = std::time::SystemTime::now();

    let result = k_convert(&array, Level::Day3);

    let x = std::time::SystemTime::now().duration_since(x).unwrap();

    println!("{}", x.as_millis());

    assert!(result[1].time - 1000 * 60 * 60 * 24 * 3 == result[2].time);
}

#[test]
fn test_k_convert13() {
    let array = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-1m.json")).unwrap();

    let x = std::time::SystemTime::now();

    let result = k_convert(&array, Level::Week1);

    let x = std::time::SystemTime::now().duration_since(x).unwrap();

    println!("{}", x.as_millis());

    assert!(result[1].time - 1000 * 60 * 60 * 24 * 7 == result[2].time);
}

#[test]
fn test_k_convert14() {
    let array = serde_json::from_str::<Vec<K>>(include_str!("../BTC-USDT-SWAP-1m.json")).unwrap();

    let x = std::time::SystemTime::now();

    let result = k_convert(&array, Level::Month1);

    let x = std::time::SystemTime::now().duration_since(x).unwrap();

    println!("{}", x.as_millis());

    let temp = chrono::NaiveDateTime::from_timestamp_millis(result[1].time as i64).unwrap();

    let next = temp - chrono::Months::new(1);

    assert!(result[2].time == next.timestamp_millis() as u64);
}

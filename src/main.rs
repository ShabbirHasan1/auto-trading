use auto_trading::*;

#[tokio::main]
async fn main() {}

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

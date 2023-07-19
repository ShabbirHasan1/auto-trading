use crate::*;

#[derive(Debug, Clone)]
pub struct Okx {
    client: reqwest::Client,
    base_url: String,
}

impl Okx {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://www.okx.com".to_string(),
        }
    }

    pub fn client(mut self, client: reqwest::Client) -> Self {
        self.client = client;
        self
    }

    pub fn base_url<S>(mut self, base_url: S) -> Self
    where
        S: AsRef<str>,
    {
        self.base_url = base_url.as_ref().to_string();
        self
    }
}

#[async_trait::async_trait]
impl Bourse for Okx {
    async fn get_k<S>(&self, product: S, level: Level, time: u64) -> anyhow::Result<Vec<K>>
    where
        S: AsRef<str>,
        S: core::marker::Send,
    {
        let level = match level {
            Level::Minute1 => "1m",
            Level::Minute5 => "m5",
            Level::Minute15 => "m15",
            Level::Minute30 => "m30",
            Level::Hour1 => "1H",
            Level::Hour4 => "4H",
            Level::Day1 => "1Dutc",
            Level::Week1 => "1Wutc",
            Level::Month1 => "1Mutc",
        };

        let mut url = self.base_url.clone();

        let args = if time != 0 {
            url += "/api/v5/market/history-index-candles";
            serde_json::json!({
                "instId": product.as_ref(),
                "bar": level,
                "after": time,
            })
        } else {
            url += "/api/v5/market/index-candles";
            serde_json::json!({
                "instId": product.as_ref(),
                "bar": level,
            })
        };

        let result = self
            .client
            .get(url)
            .query(&args)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        anyhow::ensure!(result["code"] == "0", result.to_string());

        let result = result["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| K {
                time: v[0].as_str().unwrap().parse::<u64>().unwrap(),
                open: v[1].as_str().unwrap().parse::<f64>().unwrap(),
                high: v[2].as_str().unwrap().parse::<f64>().unwrap(),
                low: v[3].as_str().unwrap().parse::<f64>().unwrap(),
                close: v[4].as_str().unwrap().parse::<f64>().unwrap(),
            })
            .collect();

        Ok(result)
    }

    async fn get_k_mark<S>(&self, product: S, level: Level, time: u64) -> anyhow::Result<Vec<K>>
    where
        S: AsRef<str>,
        S: core::marker::Send,
    {
        let level = match level {
            Level::Minute1 => "1m",
            Level::Minute5 => "m5",
            Level::Minute15 => "m15",
            Level::Minute30 => "m30",
            Level::Hour1 => "1H",
            Level::Hour4 => "4H",
            Level::Day1 => "1Dutc",
            Level::Week1 => "1Wutc",
            Level::Month1 => "1Mutc",
        };

        let mut url = self.base_url.clone();

        let args = if time != 0 {
            url += "/api/v5/market/history-mark-price-candles";
            serde_json::json!({
                "instId": product.as_ref(),
                "bar": level,
                "after": time,
            })
        } else {
            url += "/api/v5/market/mark-price-candles";
            serde_json::json!({
                "instId": product.as_ref(),
                "bar": level,
            })
        };

        let result = self
            .client
            .get(url)
            .query(&args)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        anyhow::ensure!(result["code"] == "0", result.to_string());

        let result = result["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| K {
                time: v[0].as_str().unwrap().parse::<u64>().unwrap(),
                open: v[1].as_str().unwrap().parse::<f64>().unwrap(),
                high: v[2].as_str().unwrap().parse::<f64>().unwrap(),
                low: v[3].as_str().unwrap().parse::<f64>().unwrap(),
                close: v[4].as_str().unwrap().parse::<f64>().unwrap(),
            })
            .collect();

        Ok(result)
    }

    async fn get_min_unit<S>(&self, product: S) -> anyhow::Result<f64>
    where
        S: AsRef<str>,
        S: core::marker::Send,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[tokio::test]
    async fn get_k() {
        let okx = Okx::new().base_url("https://www.rkdfs.com");
        let k1 = okx.get_k("BTC-USDT", Level::Day1, 0).await.unwrap();
        let k2 = okx
            .get_k(
                "BTC-USDT",
                Level::Day1,
                k1[k1.len() - 1].time + 1000 * 60 * 60 * 24,
            )
            .await
            .unwrap();
        assert!(k1[k1.len() - 1] == k2[0]);
    }

    #[tokio::test]
    async fn get_k_mark() {
        let okx = Okx::new().base_url("https://www.rkdfs.com");
        let k1 = okx.get_k_mark("BTC-USDT", Level::Day1, 0).await.unwrap();
        let k2 = okx
            .get_k_mark(
                "BTC-USDT",
                Level::Day1,
                k1[k1.len() - 1].time + 1000 * 60 * 60 * 24,
            )
            .await
            .unwrap();
        assert!(k1[k1.len() - 1] == k2[0]);
    }
}

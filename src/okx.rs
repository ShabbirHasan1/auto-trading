use crate::*;

#[derive(Debug, Clone)]
pub struct Okx {
    client: reqwest::Client,
    base_url: String,
}

impl Okx {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            client: reqwest::ClientBuilder::new()
                .timeout(std::time::Duration::from_secs(5))
                .build()?,
            base_url: "https://www.okx.com".to_string(),
        })
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            client,
            base_url: "https://www.okx.com".to_string(),
        }
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
        S: std::marker::Send,
    {
        let product = product.as_ref();

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
                "instId": product,
                "bar": level,
                "after": time,
            })
        } else {
            url += "/api/v5/market/index-candles";
            serde_json::json!({
                "instId": product,
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

        let array = result["data"]
            .as_array()
            .ok_or(anyhow::anyhow!("interface exception"))?;

        let mut result = Vec::with_capacity(array.len());

        for i in array {
            result.push(K {
                time: i[0]
                    .as_str()
                    .ok_or(anyhow::anyhow!("interface exception"))?
                    .parse::<u64>()?,
                open: i[1]
                    .as_str()
                    .ok_or(anyhow::anyhow!("interface exception"))?
                    .parse::<f64>()?,
                high: i[2]
                    .as_str()
                    .ok_or(anyhow::anyhow!("interface exception"))?
                    .parse::<f64>()?,
                low: i[3]
                    .as_str()
                    .ok_or(anyhow::anyhow!("interface exception"))?
                    .parse::<f64>()?,
                close: i[4]
                    .as_str()
                    .ok_or(anyhow::anyhow!("interface exception"))?
                    .parse::<f64>()?,
            });
        }

        Ok(result)
    }

    async fn get_k_mark<S>(&self, product: S, level: Level, time: u64) -> anyhow::Result<Vec<K>>
    where
        S: AsRef<str>,
        S: std::marker::Send,
    {
        let product = product.as_ref();

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
                "instId": product,
                "bar": level,
                "after": time,
            })
        } else {
            url += "/api/v5/market/index-candles";
            serde_json::json!({
                "instId": product,
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

        let array = result["data"]
            .as_array()
            .ok_or(anyhow::anyhow!("interface exception"))?;

        let mut result = Vec::with_capacity(array.len());

        for i in array {
            result.push(K {
                time: i[0]
                    .as_str()
                    .ok_or(anyhow::anyhow!("interface exception"))?
                    .parse::<u64>()?,
                open: i[1]
                    .as_str()
                    .ok_or(anyhow::anyhow!("interface exception"))?
                    .parse::<f64>()?,
                high: i[2]
                    .as_str()
                    .ok_or(anyhow::anyhow!("interface exception"))?
                    .parse::<f64>()?,
                low: i[3]
                    .as_str()
                    .ok_or(anyhow::anyhow!("interface exception"))?
                    .parse::<f64>()?,
                close: i[4]
                    .as_str()
                    .ok_or(anyhow::anyhow!("interface exception"))?
                    .parse::<f64>()?,
            });
        }

        Ok(result)
    }

    async fn get_min_unit<S>(&self, product: S) -> anyhow::Result<f64>
    where
        S: AsRef<str>,
        S: std::marker::Send,
    {
        let product = product.as_ref();

        let inst_type = if product.contains("SWAP") {
            "SWAP"
        } else {
            "SPOT"
        };

        let result = self
            .client
            .get(self.base_url.clone() + "/api/v5/public/instruments")
            .query(&serde_json::json!({
                "instType": inst_type,
                "instId": product
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        anyhow::ensure!(result["code"] == "0", result.to_string());

        Ok(if inst_type == "SWAP" {
            result["data"][0]["ctVal"]
                .as_str()
                .ok_or(anyhow::anyhow!("interface exception"))?
                .parse::<f64>()?
        } else {
            result["data"][0]["minSz"]
                .as_str()
                .ok_or(anyhow::anyhow!("interface exception"))?
                .parse::<f64>()?
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[tokio::test]
    async fn get_k() {
        let okx = Okx::new().unwrap().base_url("https://www.rkdfs.com");
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
        let okx = Okx::new().unwrap().base_url("https://www.rkdfs.com");
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

    #[tokio::test]
    async fn get_min_unit() {
        let okx = Okx::new().unwrap().base_url("https://www.rkdfs.com");
        let x = okx.get_min_unit("BTC-USDT-SWAP").await.unwrap();
        assert!(x == 0.01);
        let x = okx.get_min_unit("BTC-USDT").await.unwrap();
        assert!(x == 0.00001);
    }
}

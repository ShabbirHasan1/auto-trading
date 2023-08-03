use crate::*;

/// 交易所。
#[async_trait::async_trait]
pub trait Bourse {
    /// 获取 K 线最新价格。
    ///
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `level` 时间级别。
    /// * `time` 获取这个时间之前的数据，0 表示获取最近的数据。
    /// * `return` K 线数组。
    async fn get_k<S>(&self, product: S, level: Level, time: u64) -> anyhow::Result<Vec<K>>
    where
        S: AsRef<str>,
        S: std::marker::Send;

    /// 获取 K 线标记价格。
    ///
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `level` 时间级别。
    /// * `time` 获取这个时间之前的数据，0 表示获取最近的数据。
    /// * `return` K 线数组。
    async fn get_k_mark<S>(&self, product: S, level: Level, time: u64) -> anyhow::Result<Vec<K>>
    where
        S: AsRef<str>,
        S: std::marker::Send;

    /// 获取单笔最小交易数量。
    ///
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `return` 1张 = 价格 * 返回值
    async fn get_min_unit<S>(&self, product: S) -> anyhow::Result<f64>
    where
        S: AsRef<str>,
        S: std::marker::Send;
}

/// 本地交易所
#[derive(Debug, Clone)]
pub struct LocalBourse {
    inner: std::collections::HashMap<String, (std::collections::HashMap<Level, Vec<K>>, f64)>,
}

impl LocalBourse {
    pub fn new() -> Self {
        Self {
            inner: std::collections::HashMap::new(),
        }
    }

    /// 插入数据。
    ///
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `k` K 线数据。
    /// * `min_unit` 单笔最小交易数量。
    /// * `return` 旧值
    pub fn insert<S>(
        &mut self,
        product: S,
        level: Level,
        k: Vec<K>,
        min_unit: f64,
    ) -> Option<Vec<K>>
    where
        S: AsRef<str>,
    {
        let product = product.as_ref().to_string();
        if let Some(v) = self.inner.get_mut(&product) {
            v.1 = min_unit;
            v.0.insert(level, k)
        } else {
            todo!()
        }
    }
}

impl std::ops::Deref for LocalBourse {
    type Target =
        std::collections::HashMap<String, (std::collections::HashMap<Level, Vec<K>>, f64)>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for LocalBourse {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[async_trait::async_trait]
impl Bourse for LocalBourse {
    async fn get_k<S>(&self, product: S, level: Level, time: u64) -> anyhow::Result<Vec<K>>
    where
        S: AsRef<str>,
        S: std::marker::Send,
    {
        let product = product.as_ref().to_string();
        self.inner
            .get(&product)
            .ok_or(anyhow::anyhow!("product does not exist: {}", product))
            .and_then(|v| {
                v.0.get(&level)
                    .ok_or(anyhow::anyhow!(
                        "product does not exist: {}: {}",
                        product,
                        level
                    ))
                    .map(|v| v.iter().filter(|v| v.time <= time).cloned().collect())
            })
    }

    async fn get_k_mark<S>(&self, product: S, level: Level, time: u64) -> anyhow::Result<Vec<K>>
    where
        S: AsRef<str>,
        S: std::marker::Send,
    {
        self.get_k(product, level, time).await
    }

    async fn get_min_unit<S>(&self, product: S) -> anyhow::Result<f64>
    where
        S: AsRef<str>,
        S: std::marker::Send,
    {
        let product = product.as_ref();
        self.inner
            .get(product)
            .ok_or(anyhow::anyhow!(
                "product min unit does not exist: {}",
                product
            ))
            .map(|v| v.1)
    }
}

/// 欧易
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

        let mut result = self
            .client
            .get(&url)
            .query(&args)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        // 频繁获取数据时返回的错误代码
        while result["code"] == "50011" {
            result = self
                .client
                .get(&url)
                .query(&args)
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;
        }

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

/// 币安
#[derive(Debug, Clone)]
pub struct Binance {}

#[async_trait::async_trait]
impl crate::Bourse for Binance {
    async fn get_k<S>(
        &self,
        product: S,
        level: crate::Level,
        time: u64,
    ) -> anyhow::Result<Vec<crate::K>>
    where
        S: AsRef<str>,
        S: std::marker::Send,
    {
        todo!()
    }

    async fn get_k_mark<S>(
        &self,
        product: S,
        level: crate::Level,
        time: u64,
    ) -> anyhow::Result<Vec<crate::K>>
    where
        S: AsRef<str>,
        S: std::marker::Send,
    {
        todo!()
    }

    async fn get_min_unit<S>(&self, product: S) -> anyhow::Result<f64>
    where
        S: AsRef<str>,
        S: std::marker::Send,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[tokio::test]
    async fn okx_get_k() {
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
    async fn okx_get_k_mark() {
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
    async fn okx_get_min_unit() {
        let okx = Okx::new().unwrap().base_url("https://www.rkdfs.com");
        let x = okx.get_min_unit("BTC-USDT-SWAP").await.unwrap();
        assert!(x == 0.01);
        let x = okx.get_min_unit("BTC-USDT").await.unwrap();
        assert!(x == 0.00001);
    }
}

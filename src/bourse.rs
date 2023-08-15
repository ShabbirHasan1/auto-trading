use chrono::Datelike;

use crate::*;

/// 交易所。
#[async_trait::async_trait]
pub trait Bourse {
    /// 获取 K 线价格。
    /// 新的数据在前面。
    ///
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `level` 时间级别。
    /// * `time` 获取这个时间之前的数据，单位毫秒，0 表示获取最近的数据。
    /// * `return` K 线数组。
    async fn get_k<S>(&self, product: S, level: Level, time: u64) -> anyhow::Result<Vec<K>>
    where
        S: AsRef<str>,
        S: Send;

    /// 获取面值。
    ///
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `return` 面值，1张 = 价格 * 面值。
    async fn get_unit<S>(&self, product: S) -> anyhow::Result<f64>
    where
        S: AsRef<str>,
        S: Send;
}

/// 本地交易所。
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
    /// * `level` 时间级别。
    /// * `k` k 线数据。
    pub fn level_k<S>(&mut self, product: S, level: Level, k: Vec<K>) -> &mut Self
    where
        S: AsRef<str>,
    {
        let entry = self
            .inner
            .entry(product.as_ref().to_string())
            .or_insert((std::collections::HashMap::new(), 0.0));
        let (level_map, _) = entry;
        let k_list = level_map.entry(level).or_insert(Vec::new());
        k_list.extend(k);
        self
    }

    /// 插入数据。
    ///
    /// * `product` 交易产品，例如，现货 BTC-USDT，合约 BTC-USDT-SWAP。
    /// * `min_unit` 单笔最小交易数量。
    pub fn min_unit<S>(&mut self, product: S, min_unit: f64) -> &mut Self
    where
        S: AsRef<str>,
    {
        if let Some(entry) = self.inner.get_mut(product.as_ref()) {
            entry.1 = min_unit;
        } else {
            self.inner.insert(
                product.as_ref().to_string(),
                (std::collections::HashMap::new(), min_unit),
            );
        }
        self
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
        S: Send,
    {
        let product = product.as_ref().to_string();
        self.inner
            .get(&product)
            .map(|v| &v.0)
            .or_else(|| {
                let product = product_mapping(&product);
                self.inner.get(product.as_ref()).map(|v| &v.0)
            })
            .ok_or_else(|| anyhow::anyhow!("product does not exist: {}", product))
            .and_then(|v| {
                v.get(&level)
                    .ok_or_else(|| {
                        anyhow::anyhow!("product does not exist: {}: {}", product, level)
                    })
                    .map(|v| {
                        v.iter()
                            .filter(|v| time == 0 || v.time < time)
                            .cloned()
                            .collect()
                    })
            })
    }

    async fn get_unit<S>(&self, product: S) -> anyhow::Result<f64>
    where
        S: AsRef<str>,
        S: Send,
    {
        let product = product.as_ref();
        self.inner
            .get(product)
            .or_else(|| {
                let product = product_mapping(&product);
                self.inner.get(product.as_ref())
            })
            .ok_or(anyhow::anyhow!(
                "product min unit does not exist: {}",
                product
            ))
            .map(|v| v.1)
    }
}

/// 欧易。
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
        S: Send,
    {
        let product = product.as_ref();

        let product = if product.contains("-") {
            product.into()
        } else {
            product_mapping(product)
        };

        let (level, unit) = match level {
            Level::Minute1 => ("1m", 60 * 1000),
            Level::Minute3 => ("3m", 3 * 60 * 1000),
            Level::Minute5 => ("5m", 5 * 60 * 1000),
            Level::Minute15 => ("15m", 15 * 60 * 1000),
            Level::Minute30 => ("30m", 30 * 60 * 1000),
            Level::Hour1 => ("1H", 60 * 60 * 1000),
            Level::Hour2 => ("2H", 2 * 60 * 60 * 1000),
            Level::Hour4 => ("4H", 4 * 60 * 60 * 1000),
            Level::Hour6 => ("6Hutc", 6 * 60 * 60 * 1000),
            Level::Hour12 => ("12Hutc", 12 * 60 * 60 * 1000),
            Level::Day1 => ("1Dutc", 24 * 60 * 60 * 1000),
            Level::Day3 => ("3Dutc", 3 * 24 * 60 * 60 * 1000),
            Level::Week1 => ("1Wutc", 7 * 24 * 60 * 60 * 1000),
            Level::Month1 => {
                // 获取当前时间戳与月初时间戳的差值
                let now = chrono::Utc::now();
                let year = now.year();
                let month = now.month();
                let first_day_of_month =
                    chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, year, month, 1, 0, 0, 0)
                        .unwrap();
                let timestamp = first_day_of_month.timestamp_millis() as u64;
                (
                    "1Mutc",
                    std::time::SystemTime::now()
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64
                        - timestamp,
                )
            }
        };

        let mut url = self.base_url.clone();

        let args = if time == 0 || {
            if let Some(v) = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .checked_sub(std::time::Duration::from_millis(time))
            {
                v <= std::time::Duration::from_millis(unit)
            } else {
                false
            }
        } {
            url += "/api/v5/market/candles";
            serde_json::json!({
                "instId": product,
                "bar": level,
                "limit": "300"
            })
        } else {
            url += "/api/v5/market/history-candles";
            serde_json::json!({
                "instId": product,
                "bar": level,
                "after": time,
            })
        };

        let result = self
            .client
            .get(&url)
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

    async fn get_unit<S>(&self, product: S) -> anyhow::Result<f64>
    where
        S: AsRef<str>,
        S: Send,
    {
        let product = product.as_ref();

        let product = if product.contains("-") {
            product.into()
        } else {
            product_mapping(product)
        };

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

/// 币安。
#[derive(Debug, Clone)]
pub struct Binance {
    client: reqwest::Client,
    base_url: String,
}

impl Binance {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            client: reqwest::ClientBuilder::new()
                .timeout(std::time::Duration::from_secs(5))
                .build()?,
            base_url: "https://fapi.binance.com".to_string(),
        })
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            client,
            base_url: "https://fapi.binance.com".to_string(),
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
impl crate::Bourse for Binance {
    async fn get_k<S>(
        &self,
        product: S,
        level: crate::Level,
        time: u64,
    ) -> anyhow::Result<Vec<crate::K>>
    where
        S: AsRef<str>,
        S: Send,
    {
        let product = product.as_ref();

        let product = if product.contains("-") {
            product_mapping(product)
        } else {
            product.into()
        };

        let level = match level {
            Level::Minute1 => "1m",
            Level::Minute3 => "3m",
            Level::Minute5 => "5m",
            Level::Minute15 => "15m",
            Level::Minute30 => "30m",
            Level::Hour1 => "1h",
            Level::Hour2 => "2h",
            Level::Hour4 => "4h",
            Level::Hour6 => "6h",
            Level::Hour12 => "12h",
            Level::Day1 => "1d",
            Level::Day3 => "3d",
            Level::Week1 => "1w",
            Level::Month1 => "1M",
        };

        let mut url = self.base_url.clone();

        let args = if product.ends_with("SWAP") {
            let product = &product[0..product.len() - 4];

            url += "/fapi/v1/continuousKlines";

            if time == 0 {
                serde_json::json!({
                    "pair": product,
                    "interval": level,
                    "contractType": "PERPETUAL",
                    "limit": 1500
                })
            } else {
                serde_json::json!({
                    "pair": product,
                    "interval": level,
                    "contractType": "PERPETUAL",
                    "endTime": time - 1,
                    "limit": 1500
                })
            }
        } else {
            url += "/fapi/v1/klines";

            if time == 0 {
                serde_json::json!({
                    "symbol": product,
                    "interval": level,
                    "limit": 1500
                })
            } else {
                serde_json::json!({
                    "symbol": product,
                    "interval": level,
                    "endTime": time - 1,
                    "limit": 1500
                })
            }
        };

        let result = self
            .client
            .get(&url)
            .query(&args)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        anyhow::ensure!(result.is_array(), result.to_string());

        let array = result.as_array().unwrap();

        let mut result = Vec::with_capacity(array.len());

        for i in array.iter().rev() {
            result.push(K {
                time: i[0]
                    .as_u64()
                    .ok_or(anyhow::anyhow!("interface exception"))?,
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

    async fn get_unit<S>(&self, product: S) -> anyhow::Result<f64>
    where
        S: AsRef<str>,
        S: Send,
    {
        todo!("官方 api 返回的信息太多了，没看懂！");
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[tokio::test]
    async fn okx_get_k() {
        let bourse = Okx::new().unwrap();

        let k1 = bourse
            .get_k("BTC-USDT-SWAP", Level::Hour1, 0)
            .await
            .unwrap();

        let k2 = bourse
            .get_k("BTC-USDT-SWAP", Level::Hour1, k1.last().unwrap().time)
            .await
            .unwrap();

        println!("{}", time_to_string(k1[0].time));
        println!("{}", time_to_string(k1.last().unwrap().time));
        println!("{}", time_to_string(k2[0].time));
        println!("{}", time_to_string(k2.last().unwrap().time));

        assert!(k1.last().unwrap().time != k2[0].time);
    }

    #[tokio::test]
    async fn okx_get_min_unit() {
        let bourse = Okx::new().unwrap();
        let x = bourse.get_unit("BTC-USDT-SWAP").await.unwrap();
        assert!(x == 0.01);
        let x = bourse.get_unit("BTC-USDT").await.unwrap();
        assert!(x == 0.00001);
    }

    #[tokio::test]
    async fn binance_get_k() {
        let bourse = Binance::new().unwrap();

        let k1 = bourse
            .get_k("BTC-USDT-SWAP", Level::Hour1, 0)
            .await
            .unwrap();

        let k2 = bourse
            .get_k("BTC-USDT-SWAP", Level::Hour1, k1.last().unwrap().time)
            .await
            .unwrap();

        println!("{}", time_to_string(k1[0].time));
        println!("{}", time_to_string(k1.last().unwrap().time));
        println!("{}", time_to_string(k2[0].time));
        println!("{}", time_to_string(k2.last().unwrap().time));

        assert!(k1.last().unwrap().time != k2[0].time);
    }
}

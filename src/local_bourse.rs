use crate::*;

/// 本地交易所
pub struct LocalBoures {
    inner: std::collections::HashMap<String, (std::collections::HashMap<Level, Vec<K>>, f64)>,
}

impl LocalBoures {
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

impl std::ops::Deref for LocalBoures {
    type Target =
        std::collections::HashMap<String, (std::collections::HashMap<Level, Vec<K>>, f64)>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for LocalBoures {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[async_trait::async_trait]
impl Bourse for LocalBoures {
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

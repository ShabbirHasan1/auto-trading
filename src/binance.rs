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

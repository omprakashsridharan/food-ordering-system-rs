pub mod consumer {
    #[async_trait::async_trait]
    pub trait Consumer<T> {
        async fn receive(&self, message: T);
    }
}

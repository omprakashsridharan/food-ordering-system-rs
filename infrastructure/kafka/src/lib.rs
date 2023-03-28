pub mod consumer {
    #[async_trait::async_trait]
    pub trait Consumer<T> {
        async fn receive(&self, message: T);
    }
}

pub mod model {
    pub mod avro {
        use apache_avro::AvroSchema;

        #[derive(apache_avro::AvroSchema)]
        pub enum PaymentOrderStatus {
            PENDING,
            CANCELLED,
        }

        #[derive(AvroSchema)]
        pub struct PaymentRequest {
            pub id: uuid::Uuid,
            pub saga_id: uuid::Uuid,
            pub customer_id: uuid::Uuid,
            pub order_id: uuid::Uuid,
            pub price: f64,
            pub created_at: i64,
            pub payment_order_status: PaymentOrderStatus,
        }
    }
}

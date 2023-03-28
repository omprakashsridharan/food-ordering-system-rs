pub mod consumer {
    #[async_trait::async_trait]
    pub trait Consumer<T> {
        async fn receive(&self, message: T);
    }
}

pub mod model {
    pub mod avro {

        pub mod payment_request {
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

        pub mod payment_response {
            use apache_avro::AvroSchema;
            #[derive(apache_avro::AvroSchema)]
            pub enum PaymentStatus {
                COMPLETED,
                FAILED,
                CANCELLED,
            }

            #[derive(AvroSchema)]
            pub struct PaymentResponse {
                pub id: uuid::Uuid,
                pub saga_id: uuid::Uuid,
                pub payment_id: uuid::Uuid,
                pub customer_id: uuid::Uuid,
                pub order_id: uuid::Uuid,
                pub price: f64,
                pub created_at: i64,
                pub payment_status: PaymentStatus,
                pub failure_messages: Vec<String>,
            }
        }

        pub mod restaurant_approval_request {
            use apache_avro::AvroSchema;
            #[derive(apache_avro::AvroSchema)]
            pub enum RestaurantOrderStatus {
                PAID,
            }

            #[derive(AvroSchema)]

            pub struct Product {
                pub id: uuid::Uuid,
                pub quantity: i32,
            }
            #[derive(AvroSchema)]
            pub struct RestaurantApprovalRequest {
                pub id: uuid::Uuid,
                pub saga_id: uuid::Uuid,
                pub restaurant_id: uuid::Uuid,
                pub order_id: uuid::Uuid,
                pub restaurant_order_status: RestaurantOrderStatus,
                pub price: f64,
                pub products: Vec<Product>,
                pub created_at: i64,
            }
        }

        pub mod restaurant_approval_response {
            use apache_avro::AvroSchema;
            #[derive(apache_avro::AvroSchema)]
            pub enum OrderApprovalStatus {
                APPROVED,
                REJECTED,
            }

            #[derive(AvroSchema)]
            pub struct RestaurantApprovalResponse {
                pub id: uuid::Uuid,
                pub saga_id: uuid::Uuid,
                pub restaurant_id: uuid::Uuid,
                pub order_id: uuid::Uuid,
                pub order_approval_status: OrderApprovalStatus,
                pub created_at: i64,
                pub failure_messages: Vec<String>,
            }
        }
    }
}

pub mod error {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum KafkaError {
        #[error("ProducerError: {0}")]
        ProducerError(String),
    }
}

pub mod producer {
    pub mod service {
        use crate::error::KafkaError;

        trait Producer<K, V> {
            fn send(&self, key: K, message: V) -> Result<(), KafkaError>;
        }

        pub struct KafkaProducerImpl<K, V> {
            // producer: rdkafka::producer::FutureProducer,
            topic: String,
            _key: std::marker::PhantomData<K>,
            _value: std::marker::PhantomData<V>,
        }

        impl<K, V> Producer<K, V> for KafkaProducerImpl<K, V> {
            fn send(&self, key: K, message: V) -> Result<(), KafkaError> {
                // let message = rdkafka::producer::FutureRecord::to(&self.topic)
                //     .key(&key)
                //     .payload(&message);
                // let _ = self.producer.send(message, 0);
                // Ok(())
                todo!()
            }
        }
    }
}

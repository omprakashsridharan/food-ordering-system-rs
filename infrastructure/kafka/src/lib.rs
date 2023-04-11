use apache_avro::AvroSchema;
use serde::Serialize;

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
            use derive_builder::Builder;
            #[derive(apache_avro::AvroSchema, Clone)]
            pub enum PaymentOrderStatus {
                PENDING,
                CANCELLED,
            }

            #[derive(AvroSchema, Builder)]
            pub struct PaymentRequest {
                pub id: uuid::Uuid,
                pub saga_id: uuid::Uuid,
                pub customer_id: uuid::Uuid,
                pub order_id: uuid::Uuid,
                pub price: i64,
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
        #[error("KafkaError")]
        KafkaError(#[from] rdkafka::error::KafkaError),
    }
}

#[derive(Serialize)]
struct Message<T: AvroSchema + Serialize + Send + Sync> {
    topic: String,
    key: String,
    value: T,
}

pub mod producer {
    use apache_avro::AvroSchema;
    use serde::Serialize;

    use crate::{error::KafkaError, Message};

    #[async_trait::async_trait]
    trait KafkaProducer {
        async fn produce<T: AvroSchema + Serialize + Send + Sync>(
            &self,
            message: Message<T>,
        ) -> Result<(), KafkaError>;
    }
    pub mod service {
        use std::time::Duration;

        use apache_avro::{AvroSchema, Writer};
        use rdkafka::{
            producer::{FutureProducer, FutureRecord},
            util::Timeout,
            ClientConfig,
        };
        use serde::Serialize;

        use crate::{error::KafkaError, Message};

        use super::KafkaProducer;

        pub struct KafkaProducerImpl {
            producer: rdkafka::producer::FutureProducer,
        }

        impl KafkaProducerImpl {
            pub fn new(brokers: String, schema_registry_url: String) -> Self {
                let mut producer_config = ClientConfig::new();
                producer_config.set("bootstrap.servers", brokers);
                producer_config.set("message.timeout.ms", "5000");
                producer_config.set("schema.registry.url", schema_registry_url.as_str());
                producer_config.set(
                    "key.serializer",
                    "org.apache.kafka.common.serialization.StringSerializer",
                );
                producer_config.set(
                    "value.serializer",
                    "io.confluent.kafka.serializers.KafkaAvroSerializer",
                );
                let producer: FutureProducer =
                    producer_config.create().expect("Failed to create producer");

                KafkaProducerImpl { producer }
            }
        }

        #[async_trait::async_trait]
        impl KafkaProducer for KafkaProducerImpl {
            async fn produce<T: AvroSchema + Serialize + Send + Sync>(
                &self,
                message: Message<T>,
            ) -> Result<(), KafkaError> {
                let schema = <T as apache_avro::AvroSchema>::get_schema();
                let mut writer = Writer::new(&schema, Vec::new());
                writer.append_ser(message.value).unwrap();
                let encoded_buffer = writer.into_inner().unwrap();

                let record = FutureRecord::to(&message.topic)
                    .key(&message.key)
                    .payload(&encoded_buffer);
                let timeout: Timeout = Timeout::After(Duration::new(0, 0));
                let send_result = self
                    .producer
                    .send(record, timeout)
                    .await
                    .map_err(|e| KafkaError::KafkaError(e.0))
                    .map(|_r| {});
                send_result
            }
        }
    }
}

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
        #[error("EncoderError")]
        EncoderError(#[from] schema_registry_converter::error::SRCError),
    }
}

pub mod producer {
    pub mod service {
        use apache_avro::Schema;
        use rdkafka::producer::FutureRecord;
        use schema_registry_converter::async_impl::{
            avro::AvroEncoder, schema_registry::SrSettings,
        };

        use crate::error::KafkaError;

        trait Producer<K, V> {
            fn send(
                &self,
                topic: String,
                schema: Schema,
                key: K,
                message: V,
            ) -> Result<(), KafkaError>;
        }

        pub struct KafkaProducerImpl<'a, K, V> {
            producer: rdkafka::producer::FutureProducer,
            _key: std::marker::PhantomData<K>,
            _value: std::marker::PhantomData<V>,
            encoder: AvroEncoder<'a>,
        }

        impl<'a, K, V> KafkaProducerImpl<'a, K, V> {
            pub fn new(brokers: &str, schema_registry_url: &str) -> Self {
                let producer = rdkafka::config::ClientConfig::new()
                    .set("bootstrap.servers", brokers)
                    .set("message.timeout.ms", "5000")
                    .create()
                    .expect("Producer creation error");
                let schema_registry_settings = SrSettings::new(schema_registry_url.to_string());
                let encoder = AvroEncoder::new(schema_registry_settings);
                KafkaProducerImpl {
                    producer,
                    _key: std::marker::PhantomData,
                    _value: std::marker::PhantomData,
                    encoder,
                }
            }
        }

        impl<'a, K: Clone, V: Clone> Producer<K, V> for KafkaProducerImpl<'a, K, V> {
            fn send(
                &self,
                topic: String,
                schema: Schema,
                key: K,
                message: V,
            ) -> Result<(), KafkaError> {
                let key = key.to_owned();
                let message = message.to_owned();

                // Encode the message payload using the Schema Registry
                let encoded_payload = self
                    .encoder
                    .encode(&schema, &message)
                    .map_err(|err| KafkaError::EncoderError(err))?;

                // Create a Kafka message with the Avro-encoded payload and schema ID
                let schema_id = self
                    .encoder
                    .get_schema_id(&schema)
                    .map_err(|err| KafkaError::EncoderError(err))?;

                let message = FutureRecord::to(&topic)
                    .key(&key)
                    .payload(&encoded_payload)
                    .timestamp(chrono::Utc::now().timestamp_millis())
                    .property("avro.schema.id", &schema_id.to_string());

                // Produce the message to the Kafka topic
                self.producer
                    .send(message, Timeout::Never)
                    .map_err(|err| KafkaError::ProducerError(err))?;

                Ok(())
            }
        }
    }
}

pub mod listener {
    pub mod kafka {}
}

pub mod publisher {
    pub mod kafka {

        pub mod create_order_message_publisher {
            use common::event::publisher::DomainEventPublisher;
            use domain_core::{entity::Order, event::OrderCreated};
            use service::ports::output::message::publisher::payment::OrderCreatedPaymentRequestMessagePublisher;

            use crate::mapper;

            pub struct CreateOrderMessagePublisher<P: kafka::producer::KafkaProducer>
            where
                P: Send + Sync,
            {
                producer: P,
            }

            #[async_trait::async_trait]
            impl<P: kafka::producer::KafkaProducer> DomainEventPublisher<Order, OrderCreated>
                for CreateOrderMessagePublisher<P>
            where
                P: Send + Sync,
            {
                async fn publish(&self, event: OrderCreated) {
                    let payment_request_message =
                        mapper::DataMapper::order_created_event_to_payment_request(event);
                    // TODO: Handle error
                    let _x = self.producer.produce(payment_request_message).await;
                }
            }

            #[async_trait::async_trait]
            impl<P: kafka::producer::KafkaProducer> OrderCreatedPaymentRequestMessagePublisher
                for CreateOrderMessagePublisher<P>
            where
                P: Send + Sync,
            {
            }
        }

        pub mod cancel_order_message_publisher {
            use common::event::publisher::DomainEventPublisher;
            use domain_core::{entity::Order, event::OrderCancelled};
            use service::ports::output::message::publisher::payment::OrderCancelledPaymentRequestMessagePublisher;

            use crate::mapper;

            pub struct CancelOrderMessagePublisher<P: kafka::producer::KafkaProducer>
            where
                P: Send + Sync,
            {
                producer: P,
            }

            #[async_trait::async_trait]
            impl<P: kafka::producer::KafkaProducer> DomainEventPublisher<Order, OrderCancelled>
                for CancelOrderMessagePublisher<P>
            where
                P: Send + Sync,
            {
                async fn publish(&self, event: OrderCancelled) {
                    let payment_request_message =
                        mapper::DataMapper::order_cancelled_event_to_payment_request(event);
                    // TODO: Handle error
                    let _x = self.producer.produce(payment_request_message).await;
                }
            }

            #[async_trait::async_trait]
            impl<P: kafka::producer::KafkaProducer> OrderCancelledPaymentRequestMessagePublisher
                for CancelOrderMessagePublisher<P>
            where
                P: Send + Sync,
            {
            }
        }

        pub mod pay_order_message_publisher {

            use common::event::publisher::DomainEventPublisher;
            use domain_core::{entity::Order, event::OrderPaid};
            use service::ports::output::message::publisher::restaurant_approval::OrderPaidRestaurantRequestMessagePublisher;

            use crate::mapper;

            pub struct PayOrderMessagePublisher<P: kafka::producer::KafkaProducer>
            where
                P: Send + Sync,
            {
                producer: P,
            }

            #[async_trait::async_trait]
            impl<P: kafka::producer::KafkaProducer> DomainEventPublisher<Order, OrderPaid>
                for PayOrderMessagePublisher<P>
            where
                P: Send + Sync,
            {
                async fn publish(&self, event: OrderPaid) {
                    let restaurant_request_message =
                        mapper::DataMapper::order_paid_event_to_restaurant_request(event);
                    // TODO: Handle error
                    let _x = self.producer.produce(restaurant_request_message).await;
                }
            }

            #[async_trait::async_trait]
            impl<P: kafka::producer::KafkaProducer> OrderPaidRestaurantRequestMessagePublisher
                for PayOrderMessagePublisher<P>
            where
                P: Send + Sync,
            {
            }
        }
    }
}

pub mod mapper {
    use domain_core::event::{OrderCancelled, OrderCreated, OrderPaid};
    use kafka::{
        model::avro::{
            payment_request::{PaymentRequest, PaymentRequestBuilder},
            restaurant_approval_request::{
                Product, RestaurantApprovalRequest, RestaurantApprovalRequestBuilder,
            },
        },
        Message, MessageBuilder,
    };

    pub struct DataMapper {}

    impl DataMapper {
        pub fn order_created_event_to_payment_request(
            order_created: OrderCreated,
        ) -> Message<PaymentRequest> {
            let payment_request_message_id = uuid::Uuid::new_v4();
            // Will change later
            let saga_id = uuid::Uuid::new_v4();
            let payment_request_message = PaymentRequestBuilder::default()
                .id(payment_request_message_id)
                .saga_id(saga_id)
                .customer_id(order_created.order.clone().customer_id.into())
                .order_id(order_created.order.clone().into())
                .price(order_created.order.clone().price.into())
                .created_at(order_created.created_at.timestamp())
                .payment_order_status(
                    kafka::model::avro::payment_request::PaymentOrderStatus::PENDING,
                )
                .build()
                .unwrap();
            MessageBuilder::default()
                .topic(String::from("payment-request")) //TODO: Should change hardcoding
                .key(payment_request_message_id.to_string())
                .value(payment_request_message)
                .build()
                .unwrap()
        }

        pub fn order_cancelled_event_to_payment_request(
            order_cancelled: OrderCancelled,
        ) -> Message<PaymentRequest> {
            let payment_request_message_id = uuid::Uuid::new_v4();
            // Will change later
            let saga_id = uuid::Uuid::new_v4();
            let payment_request_message = PaymentRequestBuilder::default()
                .id(payment_request_message_id)
                .saga_id(saga_id)
                .customer_id(order_cancelled.order.clone().customer_id.into())
                .order_id(order_cancelled.order.clone().into())
                .price(order_cancelled.order.clone().price.into())
                .created_at(order_cancelled.created_at.timestamp())
                .payment_order_status(
                    kafka::model::avro::payment_request::PaymentOrderStatus::CANCELLED,
                )
                .build()
                .unwrap();
            MessageBuilder::default()
                .topic(String::from("payment-request")) //TODO: Should change hardcoding
                .key(payment_request_message_id.to_string())
                .value(payment_request_message)
                .build()
                .unwrap()
        }

        pub fn order_paid_event_to_restaurant_request(
            order_paid: OrderPaid,
        ) -> Message<RestaurantApprovalRequest> {
            let restaurant_approval_request_message_id = uuid::Uuid::new_v4();
            // Will change later
            let saga_id = uuid::Uuid::new_v4();
            let restaurant_approval_request_message = RestaurantApprovalRequestBuilder::default()
                .id(restaurant_approval_request_message_id)
                .saga_id(saga_id)
                .restaurant_id(order_paid.order.clone().restaurant_id.into())
                .order_id(order_paid.order.clone().into())
                .price(order_paid.order.clone().price.into())
                .products(
                    order_paid
                        .order
                        .clone()
                        .items
                        .into_iter()
                        .map(|item| {
                            return Product {
                                id: item.product.into(),
                                quantity: item.quantity as i64,
                            };
                        })
                        .collect(),
                )
                .created_at(order_paid.created_at.timestamp())
                .restaurant_order_status(
                    kafka::model::avro::restaurant_approval_request::RestaurantOrderStatus::PAID,
                )
                .build()
                .unwrap();
            MessageBuilder::default()
                .topic(String::from("payment-request")) //TODO: Should change hardcoding
                .key(restaurant_approval_request_message_id.to_string())
                .value(restaurant_approval_request_message)
                .build()
                .unwrap()
        }
    }
}

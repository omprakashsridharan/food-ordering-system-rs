pub mod listener {
    pub mod kafka {}
}

pub mod publisher {
    pub mod kafka {}
}

pub mod mapper {
    use domain_core::event::OrderCreated;
    use kafka::model::avro::payment_request::{PaymentRequest, PaymentRequestBuilder};

    pub struct DataMapper {}

    impl DataMapper {
        pub fn order_created_event_to_payment_request(
            order_created: OrderCreated,
        ) -> PaymentRequest {
            let payment_request_message_id = uuid::Uuid::new_v4();
            // Will change later
            let saga_id = uuid::Uuid::new_v4();
            PaymentRequestBuilder::default()
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
                .unwrap()
        }
    }
}

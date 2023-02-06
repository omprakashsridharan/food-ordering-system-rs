mod dto {
    mod create {
        use core::entity::{Product, ProductBuilder, Restaurant, RestaurantBuilder};

        use common::value_object::{
            money::{Money, ZERO},
            OrderStatus,
        };

        pub struct OrderAddress {
            street: String,
            postal_code: String,
            city: String,
        }

        pub struct OrderItem {
            product_id: uuid::Uuid,
            quantity: i32,
            price: f64,
            sub_total: f64,
        }
        pub struct CreateOrderCommand {
            customer_id: uuid::Uuid,
            restaurant_id: uuid::Uuid,
            price: f64,
            order_address: OrderAddress,
            items: Vec<OrderItem>,
        }

        impl Into<core::entity::Restaurant> for CreateOrderCommand {
            fn into(self) -> core::entity::Restaurant {
                let products: Vec<Product> = self
                    .items
                    .iter()
                    .map(|i| Product::new(i.product_id, "".to_string(), ZERO))
                    .collect();
                Restaurant::new(self.restaurant_id, products, true)
            }
        }

        pub struct CreateOrderResponse {
            order_tracking_id: uuid::Uuid,
            order_status: OrderStatus,
            message: String,
        }
    }

    mod message {
        use chrono::prelude::*;
        use common::value_object::{OrderApprovalStatus, PaymentStatus};

        pub struct PaymentResponse {
            id: String,
            saga_id: String,
            order_id: String,
            payment_id: String,
            customer_id: String,
            price: f64,
            created_at: DateTime<Utc>,
            payment_status: PaymentStatus,
            failure_messages: Vec<String>,
        }

        pub struct RestaurantApprovalResponse {
            id: String,
            saga_id: String,
            order_id: String,
            restaurant_id: String,
            created_at: DateTime<Utc>,
            order_approval_status: OrderApprovalStatus,
            failure_messages: Vec<String>,
        }
    }

    mod track {
        use common::value_object::OrderStatus;

        pub struct TrackOrderQuery {
            order_tracking_id: uuid::Uuid,
        }

        pub struct TrackOrderResponse {
            order_tracking_id: uuid::Uuid,
            order_status: OrderStatus,
            failure_messages: Vec<String>,
        }
    }
}

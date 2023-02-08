pub mod dto {
    pub mod create {
        use core::{
            entity::{
                OrderBuilder, OrderItem as OrderItemEntity, OrderItemBuilder, Product, Restaurant,
            },
            value_object::{OrderItemId, StreetAddress, StreetAddressBuilder, TrackingId},
        };

        use common::{
            entity::{BaseEntity, BaseEntityBuilder},
            value_object::{
                money::{Money, ZERO},
                CustomerId, OrderStatus, RestaurantId,
            },
        };
        use derive_builder::Builder;

        pub struct OrderAddress {
            street: String,
            postal_code: String,
            city: String,
        }

        impl Into<StreetAddress> for OrderAddress {
            fn into(self) -> StreetAddress {
                StreetAddressBuilder::default()
                    .id(uuid::Uuid::new_v4())
                    .street(self.street)
                    .city(self.city)
                    .postal_code(self.postal_code)
                    .build()
                    .unwrap()
            }
        }

        pub struct OrderItem {
            product_id: uuid::Uuid,
            quantity: u64,
            price: f64,
            sub_total: f64,
            item_id: i64,
            order_id: uuid::Uuid,
        }

        impl Into<OrderItemEntity> for OrderItem {
            fn into(self) -> OrderItemEntity {
                let product: Product = Product::new(self.product_id, "".to_string(), ZERO);
                let price = Money::new(self.price);
                let sub_total = Money::new(self.sub_total);
                let quantity = self.quantity;
                let base_entity: BaseEntity<OrderItemId> = BaseEntityBuilder::default()
                    .id(self.item_id.into())
                    .build()
                    .unwrap();
                OrderItemBuilder::default()
                    .product(product)
                    .price(price)
                    .sub_total(sub_total)
                    .quantity(quantity)
                    .order_id(self.order_id.into())
                    .base_entity(base_entity)
                    .build()
                    .unwrap()
            }
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

        impl Into<core::entity::Order> for CreateOrderCommand {
            fn into(self) -> core::entity::Order {
                let customer_id: CustomerId = self.customer_id.into();
                let restaurant_id: RestaurantId = self.restaurant_id.into();
                let delivery_address: StreetAddress = self.order_address.into();
                let price: Money = Money::new(self.price);
                let tracking_id: TrackingId = uuid::Uuid::new_v4().into();
                let order_id = uuid::Uuid::new_v4();
                let order_items: Vec<OrderItemEntity> = self
                    .items
                    .into_iter()
                    .enumerate()
                    .map(|(index, item)| {
                        let mut new_item = item;
                        new_item.order_id = order_id.into();
                        new_item.item_id = index as i64;
                        return new_item.into();
                    })
                    .collect();
                OrderBuilder::default()
                    .customer_id(customer_id)
                    .restaurant_id(restaurant_id)
                    .street_address(delivery_address)
                    .price(price)
                    .tracking_id(tracking_id)
                    .order_status(OrderStatus::Pending)
                    .items(order_items)
                    .build()
                    .unwrap()
            }
        }

        #[derive(Clone, Builder)]
        pub struct CreateOrderResponse {
            order_tracking_id: uuid::Uuid,
            order_status: OrderStatus,
            message: String,
        }
    }

    pub mod message {
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

    pub mod track {
        use common::value_object::OrderStatus;
        use derive_builder::Builder;

        pub struct TrackOrderQuery {
            order_tracking_id: uuid::Uuid,
        }

        #[derive(Clone, Builder)]
        pub struct TrackOrderResponse {
            order_tracking_id: uuid::Uuid,
            order_status: OrderStatus,
            failure_messages: Vec<String>,
        }
    }
}

pub mod ports {
    pub mod input {
        pub mod message {
            pub mod listener {
                pub mod payment {
                    use crate::dto::message::PaymentResponse;

                    pub trait PaymentResponseListener {
                        fn payment_completed(response: PaymentResponse);
                        fn payment_cancelled(response: PaymentResponse);
                    }
                }

                pub mod restaurany {
                    use crate::dto::message::RestaurantApprovalResponse;
                    pub trait RestaurantApprovalResponseMessageListener {
                        fn order_approved(response: RestaurantApprovalResponse);
                        fn order_rejected(response: RestaurantApprovalResponse);
                    }
                }
            }
        }

        pub mod service {
            use common::error::OrderDomainError;

            use crate::dto::{
                create::{CreateOrderCommand, CreateOrderResponse},
                track::{TrackOrderQuery, TrackOrderResponse},
            };

            pub trait OrderApplicationService {
                fn create_order(
                    command: CreateOrderCommand,
                ) -> Result<CreateOrderResponse, OrderDomainError>;
                fn track_order(
                    query: TrackOrderQuery,
                ) -> Result<TrackOrderResponse, OrderDomainError>;
            }
        }
    }

    pub mod output {
        pub mod message {
            pub mod publisher {
                pub mod payment {
                    use core::{
                        entity::Order,
                        event::{OrderCancelled, OrderCreated},
                    };

                    use common::event::publisher::DomainEventPublisher;

                    pub trait OrderCancelledPaymentRequestMessagePublisher:
                        DomainEventPublisher<Order, OrderCancelled>
                    {
                    }

                    pub trait OrderCreatedPaymentRequestMessagePublisher:
                        DomainEventPublisher<Order, OrderCreated>
                    {
                    }
                }

                pub mod restaurant_approval {
                    use core::{entity::Order, event::OrderPaid};

                    use common::event::publisher::DomainEventPublisher;

                    pub trait OrderPaidRestaurantRequestMessagePublisher:
                        DomainEventPublisher<Order, OrderPaid>
                    {
                    }
                }
            }
        }

        pub mod repository {
            use core::{
                entity::{Customer, Order, Restaurant},
                value_object::TrackingId,
            };

            pub trait OrderRepository {
                fn save(order: Order) -> (bool, Order);
                fn find_by_tracking_id(id: TrackingId) -> (bool, Order);
            }

            pub trait CustomerRepository {
                fn find_customer(customer_id: uuid::Uuid) -> (bool, Customer);
            }

            pub trait RestaurantRepository {
                fn find_restaurant_info(restaurant: Restaurant) -> (bool, Restaurant);
            }
        }
    }
}

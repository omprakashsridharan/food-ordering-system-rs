use domain_core::{
    entity::{Order, Restaurant},
    event::OrderCreated,
    value_object::TrackingId,
    OrderDomainService,
};

use common::error::OrderDomainError;
use dto::{
    create::{CreateOrderCommand, CreateOrderResponse},
    message::RestaurantApprovalResponse,
    track::{TrackOrderQuery, TrackOrderResponse},
};
use ports::{
    input::{
        message::listener::{
            payment::PaymentResponseListener, restaurant::RestaurantApprovalResponseMessageListener,
        },
        service::OrderApplicationService,
    },
    output::{
        message::publisher::payment::OrderCreatedPaymentRequestMessagePublisher,
        repository::{CustomerRepository, OrderRepository, RestaurantRepository},
    },
};

pub mod dto {
    pub mod create {
        use domain_core::{
            entity::{
                Order, OrderBuilder, OrderItem as OrderItemEntity, OrderItemBuilder, Product,
                Restaurant,
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

        #[derive(Clone)]
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

        #[derive(Clone)]
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

        #[derive(Clone)]
        pub struct CreateOrderCommand {
            pub customer_id: uuid::Uuid,
            pub restaurant_id: uuid::Uuid,
            pub price: f64,
            pub order_address: OrderAddress,
            pub items: Vec<OrderItem>,
        }

        impl Into<domain_core::entity::Restaurant> for CreateOrderCommand {
            fn into(self) -> domain_core::entity::Restaurant {
                let products: Vec<Product> = self
                    .items
                    .iter()
                    .map(|i| Product::new(i.product_id, "".to_string(), ZERO))
                    .collect();
                Restaurant::new(self.restaurant_id, products, true)
            }
        }

        impl Into<domain_core::entity::Order> for CreateOrderCommand {
            fn into(self) -> domain_core::entity::Order {
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

        impl From<Order> for CreateOrderResponse {
            fn from(o: Order) -> Self {
                Self {
                    order_tracking_id: o.tracking_id.into(),
                    order_status: o.order_status,
                    message: "Order created successfully".to_string(),
                }
            }
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
        use domain_core::entity::Order;

        pub struct TrackOrderQuery {
            pub order_tracking_id: uuid::Uuid,
        }

        #[derive(Clone, Builder)]
        pub struct TrackOrderResponse {
            order_tracking_id: uuid::Uuid,
            order_status: OrderStatus,
            failure_messages: Vec<String>,
        }

        impl From<Order> for TrackOrderResponse {
            fn from(o: Order) -> Self {
                Self {
                    order_tracking_id: o.tracking_id.into(),
                    order_status: o.order_status,
                    failure_messages: o.failure_messages,
                }
            }
        }
    }
}

pub mod ports {
    pub mod input {
        pub mod message {
            pub mod listener {
                pub mod payment {
                    use crate::dto::message::PaymentResponse;

                    #[async_trait::async_trait]
                    pub trait PaymentResponseListener {
                        async fn payment_completed(&self, response: PaymentResponse);
                        async fn payment_cancelled(&self, response: PaymentResponse);
                    }
                }

                pub mod restaurant {
                    use crate::dto::message::RestaurantApprovalResponse;

                    #[async_trait::async_trait]
                    pub trait RestaurantApprovalResponseMessageListener {
                        async fn order_approved(&self, response: RestaurantApprovalResponse);
                        async fn order_rejected(&self, response: RestaurantApprovalResponse);
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

            #[async_trait::async_trait]
            pub trait OrderApplicationService: std::marker::Sync {
                async fn create_order(
                    &self,
                    command: CreateOrderCommand,
                ) -> Result<CreateOrderResponse, OrderDomainError>;
                async fn track_order(
                    &self,
                    query: TrackOrderQuery,
                ) -> Result<TrackOrderResponse, OrderDomainError>;
            }
        }
    }

    pub mod output {
        pub mod message {
            pub mod publisher {
                pub mod payment {
                    use domain_core::{
                        entity::Order,
                        event::{OrderCancelled, OrderCreated},
                    };

                    use common::event::publisher::DomainEventPublisher;

                    pub trait OrderCancelledPaymentRequestMessagePublisher:
                        DomainEventPublisher<Order, OrderCancelled>
                    {
                    }

                    #[async_trait::async_trait]
                    pub trait OrderCreatedPaymentRequestMessagePublisher:
                        DomainEventPublisher<Order, OrderCreated>
                    {
                    }
                }

                pub mod restaurant_approval {
                    use domain_core::{entity::Order, event::OrderPaid};

                    use common::event::publisher::DomainEventPublisher;

                    pub trait OrderPaidRestaurantRequestMessagePublisher:
                        DomainEventPublisher<Order, OrderPaid>
                    {
                    }
                }
            }
        }

        pub mod repository {
            use common::error::OrderDomainError;
            use domain_core::{
                entity::{Customer, Order, Restaurant},
                value_object::TrackingId,
            };

            #[async_trait::async_trait]
            pub trait OrderRepository: Send + Sync {
                async fn save(&self, order: Order) -> Result<Order, OrderDomainError>;
                async fn find_by_tracking_id(&self, id: TrackingId) -> (bool, Order);
            }

            #[async_trait::async_trait]
            pub trait CustomerRepository: Send + Sync {
                async fn find_customer(&self, customer_id: uuid::Uuid) -> (bool, Customer);
            }

            #[async_trait::async_trait]
            pub trait RestaurantRepository: Send + Sync {
                async fn find_restaurant_info(&self, restaurant: Restaurant) -> (bool, Restaurant);
            }
        }
    }
}

pub struct OrderCreateHelper<
    ODS: OrderDomainService,
    OR: OrderRepository,
    CR: CustomerRepository,
    RR: RestaurantRepository,
> {
    order_domain_service: ODS,
    order_repository: OR,
    customer_repository: CR,
    restaurant_repository: RR,
}

impl<
        ODS: OrderDomainService,
        OR: OrderRepository,
        CR: CustomerRepository,
        RR: RestaurantRepository,
    > OrderCreateHelper<ODS, OR, CR, RR>
{
    pub async fn persist_order(
        &self,
        command: CreateOrderCommand,
    ) -> Result<OrderCreated, OrderDomainError> {
        self.check_customer(command.customer_id).await?;
        let restaurant = self.check_restaurant(command.clone()).await?;
        let order: Order = command.into();
        let order_created_event = self
            .order_domain_service
            .validate_and_initiate_order(order, restaurant)?;
        Ok(order_created_event)
    }

    pub async fn check_customer(&self, customer_id: uuid::Uuid) -> Result<(), OrderDomainError> {
        let (ok, _) = self.customer_repository.find_customer(customer_id).await;
        if !ok {
            return Err(OrderDomainError::CustomerNotFound);
        } else {
            Ok(())
        }
    }

    pub async fn check_restaurant(
        &self,
        command: CreateOrderCommand,
    ) -> Result<Restaurant, OrderDomainError> {
        let restaurant: Restaurant = command.into();
        let (ok, _) = self
            .restaurant_repository
            .find_restaurant_info(restaurant.clone())
            .await;
        if !ok {
            return Err(OrderDomainError::RestaurantNotFound);
        } else {
            Ok(restaurant)
        }
    }

    pub async fn save_order(&self, order: Order) -> Result<Order, OrderDomainError> {
        self.order_repository.save(order).await
    }
}

pub struct OrderCreateCommandHandler<
    OCPRMP: OrderCreatedPaymentRequestMessagePublisher,
    ODS: OrderDomainService,
    OR: OrderRepository,
    CR: CustomerRepository,
    RR: RestaurantRepository,
> {
    order_create_helper: OrderCreateHelper<ODS, OR, CR, RR>,
    order_created_payment_request_message_publisher: OCPRMP,
}

impl<
        OCPRMP: OrderCreatedPaymentRequestMessagePublisher,
        ODS: OrderDomainService,
        OR: OrderRepository,
        CR: CustomerRepository,
        RR: RestaurantRepository,
    > OrderCreateCommandHandler<OCPRMP, ODS, OR, CR, RR>
{
    pub async fn create_order(
        &self,
        command: CreateOrderCommand,
    ) -> Result<CreateOrderResponse, OrderDomainError> {
        let order_created_event = self.order_create_helper.persist_order(command).await?;
        self.order_created_payment_request_message_publisher
            .publish(order_created_event.clone())
            .await;
        let create_order_response: CreateOrderResponse = order_created_event.0.into();
        Ok(create_order_response)
    }
}

pub struct OrderTrackCommandHandler<OR: OrderRepository> {
    order_repository: OR,
}

impl<OR: OrderRepository> OrderTrackCommandHandler<OR> {
    pub async fn track_order(
        &self,
        query: TrackOrderQuery,
    ) -> Result<TrackOrderResponse, OrderDomainError> {
        let tracking_id: TrackingId = query.order_tracking_id.into();
        let (ok, order) = self.order_repository.find_by_tracking_id(tracking_id).await;
        if !ok {
            Err(OrderDomainError::OrderNotFound)
        } else {
            Ok(order.into())
        }
    }
}

pub struct OrderApplicationServiceImpl<
    OCPRMP: OrderCreatedPaymentRequestMessagePublisher,
    ODS: OrderDomainService,
    OR: OrderRepository,
    CR: CustomerRepository,
    RR: RestaurantRepository,
> {
    order_create_command_helper: OrderCreateCommandHandler<OCPRMP, ODS, OR, CR, RR>,
    order_track_comman_helper: OrderTrackCommandHandler<OR>,
}

#[async_trait::async_trait]
impl<
        OCPRMP: OrderCreatedPaymentRequestMessagePublisher,
        ODS: OrderDomainService,
        OR: OrderRepository,
        CR: CustomerRepository,
        RR: RestaurantRepository,
    > OrderApplicationService for OrderApplicationServiceImpl<OCPRMP, ODS, OR, CR, RR>
{
    async fn create_order(
        &self,
        command: dto::create::CreateOrderCommand,
    ) -> Result<dto::create::CreateOrderResponse, common::error::OrderDomainError> {
        self.order_create_command_helper.create_order(command).await
    }

    async fn track_order(
        &self,
        query: dto::track::TrackOrderQuery,
    ) -> Result<dto::track::TrackOrderResponse, common::error::OrderDomainError> {
        self.order_track_comman_helper.track_order(query).await
    }
}

pub struct PaymentResponseMessageListenerImpl {}

#[async_trait::async_trait]
impl PaymentResponseListener for PaymentResponseMessageListenerImpl {
    async fn payment_completed(&self, response: dto::message::PaymentResponse) {
        todo!()
    }

    async fn payment_cancelled(&self, response: dto::message::PaymentResponse) {
        todo!()
    }
}

pub struct RestaurantApprovalResponseMessageListenerImpl {}

#[async_trait::async_trait]
impl RestaurantApprovalResponseMessageListener for RestaurantApprovalResponseMessageListenerImpl {
    async fn order_approved(&self, response: RestaurantApprovalResponse) {}
    async fn order_rejected(&self, response: RestaurantApprovalResponse) {}
}

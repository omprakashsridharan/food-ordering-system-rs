use common::error::OrderDomainError;
use entity::{Order, Restaurant};
use event::{OrderCancelledBuilder, OrderCreatedBuilder, OrderPaidBuilder};

pub mod entity {
    use common::entity::{AggregateRoot, BaseEntity, BaseEntityBuilder};
    use common::error::OrderDomainError;
    use common::value_object::money::Money;
    use common::value_object::{
        BaseId, BaseIdBuilder, CustomerId, OrderId, OrderStatus, ProductId, RestaurantId,
        RestaurantIdBuilder,
    };
    use derive_builder::Builder;

    use crate::value_object::{OrderItemId, StreetAddress, TrackingId};

    #[derive(Clone, Builder)]
    pub struct Customer {
        pub aggregate_root: AggregateRoot<CustomerId>,
    }

    impl Into<uuid::Uuid> for Customer {
        fn into(self) -> uuid::Uuid {
            return self.aggregate_root.base_entity.id.base_id.value;
        }
    }

    #[derive(Clone, Builder)]
    pub struct Product {
        pub base_entity: BaseEntity<ProductId>,
        pub name: String,
        pub price: Money,
    }

    impl Product {
        pub fn new(product_id: uuid::Uuid, name: String, price: Money) -> Self {
            let product_id: ProductId = product_id.into();
            let base_entity: BaseEntity<ProductId> =
                BaseEntityBuilder::default().id(product_id).build().unwrap();
            Self {
                base_entity,
                name,
                price,
            }
        }
    }

    impl Into<uuid::Uuid> for Product {
        fn into(self) -> uuid::Uuid {
            return self.base_entity.id.base_id.value;
        }
    }

    impl PartialEq for Product {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name && self.price == other.price
        }
    }

    #[derive(Clone, Builder)]
    pub struct Restaurant {
        pub base_entity: BaseEntity<RestaurantId>,
        pub products: Vec<Product>,
        pub active: bool,
    }

    impl Into<uuid::Uuid> for Restaurant {
        fn into(self) -> uuid::Uuid {
            return self.base_entity.id.base_id.value;
        }
    }

    impl Restaurant {
        pub fn is_active(&self) -> bool {
            return self.active;
        }

        pub fn new(restaurant_id: uuid::Uuid, products: Vec<Product>, active: bool) -> Self {
            let base_id: BaseId<uuid::Uuid> = BaseIdBuilder::default()
                .value(restaurant_id)
                .build()
                .unwrap();
            let restaurant_id: RestaurantId = RestaurantIdBuilder::default()
                .base_id(base_id)
                .build()
                .unwrap();
            let base_entity: BaseEntity<RestaurantId> = BaseEntityBuilder::default()
                .id(restaurant_id)
                .build()
                .unwrap();
            Self {
                base_entity,
                products,
                active,
            }
        }
    }

    #[derive(Clone, Builder)]
    pub struct OrderItem {
        pub base_entity: BaseEntity<OrderItemId>,
        pub order_id: OrderId,
        pub product: Product,
        quantity: u64,
        price: Money,
        sub_total: Money,
    }

    impl OrderItem {
        pub fn is_price_valid(&self) -> bool {
            return self.price.is_greater_than_zero()
                && self.price == self.product.price
                && self.price.clone() * self.quantity == self.sub_total;
        }
    }

    #[derive(Clone, Builder)]
    pub struct Order {
        pub aggregate_root: AggregateRoot<OrderId>,
        pub customer_id: CustomerId,
        pub restaurant_id: RestaurantId,
        pub street_address: StreetAddress,
        pub price: Money,
        pub items: Vec<OrderItem>,
        pub tracking_id: TrackingId,
        pub order_status: OrderStatus,
        pub failure_messages: Vec<String>,
    }

    impl Into<uuid::Uuid> for Order {
        fn into(self) -> uuid::Uuid {
            return self.aggregate_root.base_entity.id.base_id.value;
        }
    }

    impl Order {
        pub fn validate_total_price(&self) -> Result<(), OrderDomainError> {
            if self.price.is_greater_than_zero() {
                return Err(OrderDomainError::TotalPriceZeroError);
            }
            return Ok(());
        }

        pub fn validate_items_price(&self) -> Result<(), OrderDomainError> {
            let mut order_items_total_price = Money::new(0.0);
            for item in self.items.iter() {
                if item.is_price_valid() {
                    order_items_total_price += item.sub_total.clone();
                } else {
                    return Err(OrderDomainError::OrderItemPriceInvalid);
                }
            }
            if order_items_total_price != self.price {
                return Err(OrderDomainError::OrderTotalPriceMismatch);
            }
            return Ok(());
        }

        pub fn validate_order(&self) -> Result<(), OrderDomainError> {
            self.validate_total_price()?;
            self.validate_items_price()?;
            Ok(())
        }

        pub fn pay(&mut self) -> Result<(), OrderDomainError> {
            if self.order_status != OrderStatus::Pending {
                return Err(OrderDomainError::InvalidOrderStatus(String::from("pay")));
            }
            self.order_status = OrderStatus::Paid;
            Ok(())
        }

        pub fn approve(&mut self) -> Result<(), OrderDomainError> {
            if self.order_status != OrderStatus::Paid {
                return Err(OrderDomainError::InvalidOrderStatus(String::from(
                    "approve",
                )));
            }
            self.order_status = OrderStatus::Approved;
            Ok(())
        }

        pub fn init_cancel(
            &mut self,
            failure_messages: Vec<String>,
        ) -> Result<(), OrderDomainError> {
            if self.order_status != OrderStatus::Paid {
                return Err(OrderDomainError::InvalidOrderStatus(String::from(
                    "init cancel",
                )));
            }
            self.order_status = OrderStatus::Cancelling;
            self.failure_messages.append(&mut failure_messages.clone());
            Ok(())
        }

        pub fn cancel(&mut self, failure_messages: Vec<String>) -> Result<(), OrderDomainError> {
            if !(self.order_status == OrderStatus::Cancelling
                || self.order_status == OrderStatus::Pending)
            {
                return Err(OrderDomainError::InvalidOrderStatus(String::from("cancel")));
            }
            self.order_status = OrderStatus::Cancelled;
            self.failure_messages.append(&mut failure_messages.clone());
            Ok(())
        }
    }
}

pub mod value_object {
    use common::value_object::{BaseId, BaseIdBuilder};
    use derive_builder::Builder;

    #[derive(Clone, Builder)]
    pub struct OrderItemId {
        pub base_id: BaseId<i64>,
    }

    impl From<i64> for OrderItemId {
        fn from(id: i64) -> Self {
            return OrderItemIdBuilder::default()
                .base_id(BaseIdBuilder::default().value(id).build().unwrap())
                .build()
                .unwrap();
        }
    }

    #[derive(Clone, Builder)]
    pub struct StreetAddress {
        pub id: uuid::Uuid,
        pub street: String,
        pub postal_code: String,
        pub city: String,
    }

    #[derive(Clone)]
    pub struct TrackingId {
        base_id: BaseId<uuid::Uuid>,
    }

    impl From<uuid::Uuid> for TrackingId {
        fn from(id: uuid::Uuid) -> Self {
            return Self {
                base_id: BaseIdBuilder::default().value(id).build().unwrap(),
            };
        }
    }

    impl Into<uuid::Uuid> for TrackingId {
        fn into(self) -> uuid::Uuid {
            return self.base_id.value;
        }
    }
}

pub mod event {

    use common::event::DomainEvent;
    use derive_builder::Builder;

    use crate::entity::Order;

    #[derive(Clone, Builder)]
    pub struct OrderCreated {
        pub order: Order,
        pub created_at: chrono::DateTime<chrono::Utc>,
    }

    impl DomainEvent<Order> for OrderCreated {}

    #[derive(Clone, Builder)]
    pub struct OrderCancelled {
        pub order: Order,
        pub created_at: chrono::DateTime<chrono::Utc>,
    }

    impl DomainEvent<Order> for OrderCancelled {}

    #[derive(Clone, Builder)]
    pub struct OrderPaid {
        pub order: Order,
        pub created_at: chrono::DateTime<chrono::Utc>,
    }

    impl DomainEvent<Order> for OrderPaid {}
}

pub trait OrderDomainService: Send + Sync {
    fn validate_and_initiate_order(
        &self,
        order: Order,
        restaurant: Restaurant,
    ) -> Result<event::OrderCreated, OrderDomainError>;

    fn pay_order(order: Order) -> Result<event::OrderPaid, OrderDomainError>;

    fn approve_order(order: Order) -> Result<(), OrderDomainError>;

    fn cancel_order_payment(
        order: Order,
        failure_messages: Vec<String>,
    ) -> Result<event::OrderCancelled, OrderDomainError>;

    fn cancel_order(order: Order, failure_messages: Vec<String>) -> Result<(), OrderDomainError>;
}

pub struct OrderDomainServiceImpl {}

impl OrderDomainService for OrderDomainServiceImpl {
    fn validate_and_initiate_order(
        &self,
        mut order: Order,
        restaurant: Restaurant,
    ) -> Result<event::OrderCreated, OrderDomainError> {
        if !restaurant.is_active() {
            return Err(OrderDomainError::InactiveRestaurant);
        } else {
            for item in order.items.iter_mut() {
                for product in restaurant.products.iter() {
                    if *product == item.product {
                        item.product.name = product.name.clone();
                        item.product.price = product.price.clone();
                    }
                }
            }
            order.validate_order()?;
            let order_created_event = OrderCreatedBuilder::default().order(order).build().unwrap();
            Ok(order_created_event)
        }
    }

    fn pay_order(mut order: Order) -> Result<event::OrderPaid, OrderDomainError> {
        order.pay()?;
        Ok(OrderPaidBuilder::default().order(order).build().unwrap())
    }

    fn approve_order(mut order: Order) -> Result<(), OrderDomainError> {
        order.approve()?;
        Ok(())
    }

    fn cancel_order_payment(
        mut order: Order,
        failure_messages: Vec<String>,
    ) -> Result<event::OrderCancelled, OrderDomainError> {
        order.init_cancel(failure_messages)?;
        Ok(OrderCancelledBuilder::default()
            .order(order)
            .build()
            .unwrap())
    }

    fn cancel_order(
        mut order: Order,
        failure_messages: Vec<String>,
    ) -> Result<(), OrderDomainError> {
        order.cancel(failure_messages.clone())
    }
}

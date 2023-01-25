mod entity {
    use common::entity::{AggregateRoot, BaseEntity};
    use common::value_object::money::Money;
    use common::value_object::{CustomerId, OrderId, OrderStatus, ProductId, RestaurantId};

    use crate::value_object::{OrderItemId, StreetAddress, TrackingId};

    #[derive(Clone)]
    pub struct Customer {
        aggregate_root: AggregateRoot<CustomerId>,
    }

    #[derive(Clone)]
    pub struct Product {
        base_entity: BaseEntity<ProductId>,
        name: String,
        price: Money,
    }

    #[derive(Clone)]
    pub struct Restaurant {
        base_entity: BaseEntity<RestaurantId>,
        products: Vec<Product>,
        active: bool,
    }

    #[derive(Clone)]
    pub struct OrderItem {
        base_entity: BaseEntity<OrderItemId>,
        order_id: OrderId,
        product: Product,
        quantity: u64,
        price: Money,
        sub_total: Money,
    }

    #[derive(Clone)]
    pub struct Order {
        aggregate_root: AggregateRoot<OrderId>,
        customer_id: CustomerId,
        restaurant_id: RestaurantId,
        street_address: StreetAddress,
        price: Money,
        items: Vec<OrderItem>,
        tracking_id: TrackingId,
        order_status: OrderStatus,
        failure_messages: Vec<String>,
    }
}

mod value_object {
    use common::value_object::BaseId;

    #[derive(Clone)]
    pub struct OrderItemId {
        base_id: BaseId<i64>,
    }

    #[derive(Clone)]
    pub struct StreetAddress {
        id: uuid::Uuid,
        street: String,
        postal_code: String,
        city: String,
    }

    #[derive(Clone)]
    pub struct TrackingId {
        base_id: BaseId<uuid::Uuid>,
    }
}

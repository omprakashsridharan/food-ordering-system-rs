pub mod adapter {}
pub mod entity {

    pub mod order {
        use common::entity::{AggregateRoot, AggregateRootBuilder, BaseEntityBuilder};
        use common::value_object::money::Money;
        use common::value_object::{CustomerId, OrderId, OrderStatus, RestaurantId};
        use domain_core::entity::{Order, OrderBuilder};
        use domain_core::value_object::TrackingId;
        use sea_orm::entity::prelude::*;

        use sea_orm::DeriveEntityModel;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "orders")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: uuid::Uuid,
            pub customer_id: uuid::Uuid,
            pub restaurant_id: uuid::Uuid,
            pub tracking_id: uuid::Uuid,
            pub price: i64,
            pub order_status: String,
            pub failure_messages: String,
        }

        impl From<Order> for Model {
            fn from(o: Order) -> Self {
                Self {
                    id: o.clone().into(),
                    customer_id: o.customer_id.into(),
                    restaurant_id: o.restaurant_id.into(),
                    tracking_id: o.tracking_id.into(),
                    price: o.price.amount as i64,
                    order_status: o.order_status.to_string(),
                    failure_messages: o.failure_messages.join("#"),
                }
            }
        }

        impl Into<Order> for Model {
            fn into(self) -> Order {
                let order_id: OrderId = self.id.into();
                let base_entity = BaseEntityBuilder::default().id(order_id).build().unwrap();
                let aggregate_root: AggregateRoot<OrderId> = AggregateRootBuilder::default()
                    .base_entity(base_entity)
                    .build()
                    .unwrap();
                let customer_id: CustomerId = self.customer_id.into();
                let restaurant_id: RestaurantId = self.restaurant_id.into();
                let tracking_id: TrackingId = self.tracking_id.into();
                let order_status: OrderStatus = self.order_status.parse().unwrap();
                let price: Money = self.price.into();
                let failure_messages: Vec<String> = self
                    .failure_messages
                    .split("#")
                    .map(|s| s.to_string())
                    .collect();
                OrderBuilder::default()
                    .aggregate_root(aggregate_root)
                    .customer_id(customer_id)
                    .restaurant_id(restaurant_id)
                    .tracking_id(tracking_id)
                    .price(price)
                    .order_status(order_status)
                    .failure_messages(failure_messages)
                    .build()
                    .unwrap()
            }
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {
            #[sea_orm(has_one = "super::order_address::Entity")]
            OrderAddress,
            #[sea_orm(has_many = "super::order_item::Entity")]
            OrderItem,
        }

        impl Related<super::order_address::Entity> for Entity {
            fn to() -> RelationDef {
                Relation::OrderAddress.def()
            }
        }

        impl Related<super::order_item::Entity> for Entity {
            fn to() -> RelationDef {
                Relation::OrderItem.def()
            }
        }

        impl ActiveModelBehavior for ActiveModel {}
    }

    pub mod order_address {
        use sea_orm::entity::prelude::*;

        use sea_orm::DeriveEntityModel;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "order_addresses")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: uuid::Uuid,
            pub order_id: uuid::Uuid,
            pub street: String,
            pub postal_code: String,
            pub city: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {
            #[sea_orm(
                belongs_to = "super::order::Entity",
                from = "Column::OrderId",
                to = "super::order::Column::Id"
            )]
            Order,
        }

        impl Related<super::order::Entity> for Entity {
            fn to() -> RelationDef {
                Relation::Order.def()
            }
        }

        impl ActiveModelBehavior for ActiveModel {}
    }

    pub mod order_item {
        use common::entity::{BaseEntity, BaseEntityBuilder};
        use common::value_object;
        use domain_core::entity::{OrderItem, OrderItemBuilder, Product, ProductBuilder};
        use domain_core::value_object::OrderItemId;
        use sea_orm::entity::prelude::*;

        use sea_orm::DeriveEntityModel;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "order_items")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i64,
            #[sea_orm(primary_key)]
            pub order_id: uuid::Uuid,
            pub product_id: uuid::Uuid,
            pub quantity: u64,
            pub price: i64,
            pub sub_total: i64,
        }

        impl From<Model> for OrderItem {
            fn from(val: Model) -> Self {
                let order_item_id = val.id;
                let order_id: uuid::Uuid = val.order_id;
                let product_id: uuid::Uuid = val.product_id;
                let quantity: u64 = val.quantity;
                let price: i64 = val.price;
                let sub_total: i64 = val.sub_total;
                let order_item_base_entity: BaseEntity<OrderItemId> = BaseEntityBuilder::default()
                    .id(order_item_id.into())
                    .build()
                    .unwrap();
                let product_base_entity: BaseEntity<value_object::ProductId> =
                    BaseEntityBuilder::default()
                        .id(product_id.into())
                        .build()
                        .unwrap();
                let product: Product = ProductBuilder::default()
                    .base_entity(product_base_entity)
                    .build()
                    .unwrap();
                OrderItemBuilder::default()
                    .base_entity(order_item_base_entity)
                    .order_id(order_id.into())
                    .quantity(quantity)
                    .price(price.into())
                    .sub_total(sub_total.into())
                    .product(product)
                    .build()
                    .unwrap()
            }
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {
            #[sea_orm(
                belongs_to = "super::order::Entity",
                from = "Column::OrderId",
                to = "super::order::Column::Id"
            )]
            Order,
        }

        impl Related<super::order::Entity> for Entity {
            fn to() -> RelationDef {
                Relation::Order.def()
            }
        }

        impl ActiveModelBehavior for ActiveModel {}
    }
}
pub mod repository {
    use common::error::OrderDomainError;
    use domain_core::{
        entity::{Order, OrderItem},
        value_object::{StreetAddress, StreetAddressBuilder, TrackingId},
    };
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
    use service::ports::output::repository::OrderRepository;

    use crate::entity::{order, order_address, order_item};

    pub struct OrderRepositoryImpl {
        db: sea_orm::DatabaseConnection,
    }

    #[async_trait::async_trait]
    impl OrderRepository for OrderRepositoryImpl {
        async fn save(&self, order: Order) -> Result<Order, OrderDomainError> {
            let order_model: order::Model = order.clone().into();
            let order_active_model: order::ActiveModel = order_model.into();
            let _save_result = order_active_model
                .insert(&self.db)
                .await
                .map_err(|_| OrderDomainError::SaveOrderError)?;
            Ok(order)
        }
        async fn find_by_tracking_id(&self, id: TrackingId) -> Result<Order, OrderDomainError> {
            let tracking_uuid: uuid::Uuid = id.into();
            let (order_model, street_model_optional) = order::Entity::find()
                .filter(order::Column::TrackingId.eq(tracking_uuid))
                .find_also_related(order_address::Entity)
                .one(&self.db)
                .await
                .map_err(|_| OrderDomainError::OrderNotFound)?
                .ok_or(OrderDomainError::OrderNotFound)?;
            let order_item_models = order_item::Entity::find()
                .filter(order_item::Column::OrderId.eq(order_model.id))
                .all(&self.db)
                .await
                .map_err(|_| OrderDomainError::OrderItemNotFound)?;
            let street_address_model: order_address::Model = street_model_optional.unwrap();
            let street_address: StreetAddress = StreetAddressBuilder::default()
                .city(street_address_model.city)
                .postal_code(street_address_model.postal_code)
                .street(street_address_model.street)
                .id(street_address_model.id.into())
                .build()
                .unwrap();
            let order_items: Vec<OrderItem> = order_item_models
                .into_iter()
                .map(|order_item_model| order_item_model.into())
                .collect();
            let mut order: Order = order_model.into();
            order.street_address = street_address;
            order.items = order_items;
            Ok(order)
        }
    }
}

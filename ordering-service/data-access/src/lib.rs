pub mod adapter {}
pub mod entity {

    pub mod order {
        use domain_core::entity::Order;
        use sea_orm::entity::prelude::*;

        use sea_orm::DeriveEntityModel;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "orders")]
        pub struct Model {
            #[sea_orm(primary_key)]
            id: uuid::Uuid,
            customer_id: uuid::Uuid,
            restaurant_id: uuid::Uuid,
            tracking_id: uuid::Uuid,
            price: i64,
            order_status: String,
            failure_messages: String,
        }

        impl From<Order> for Model {
            fn from(o: Order) -> Self {
                Self {
                    id: o.into(),
                    customer_id: o.customer_id.into(),
                    restaurant_id: o.restaurant_id.into(),
                    tracking_id: o.tracking_id.into(),
                    price: o.price.amount as i64,
                    order_status: o.order_status.to_string(),
                    failure_messages: o.failure_messages.join("#"),
                }
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
            id: uuid::Uuid,
            order_id: uuid::Uuid,
            street: String,
            postal_code: String,
            city: String,
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
        use sea_orm::entity::prelude::*;

        use sea_orm::DeriveEntityModel;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "order_items")]
        pub struct Model {
            #[sea_orm(primary_key)]
            id: uuid::Uuid,
            #[sea_orm(primary_key)]
            order_id: uuid::Uuid,
            quantity: i32,
            price: i32,
            sub_total: i32,
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
    use domain_core::entity::Order;
    use sea_orm::ActiveModelTrait;
    use service::ports::output::repository::OrderRepository;

    use crate::entity::order;

    pub struct OrderRepositoryImpl {
        db: sea_orm::DatabaseConnection,
    }

    #[async_trait::async_trait]
    impl OrderRepository for OrderRepositoryImpl {
        async fn save(&self, order: Order) -> Result<Order, OrderDomainError> {
            let order_model: order::Model = order.into();
            let order_active_model: order::ActiveModel = order_model.into();
            let save_result = order_active_model
                .insert(&self.db)
                .await
                .map_err(|_| OrderDomainError::SaveOrderError)?;
            Ok(order)
        }
        async fn find_by_tracking_id(&self, id: TrackingId) -> (bool, Order);
    }
}

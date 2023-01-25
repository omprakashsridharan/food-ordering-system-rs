pub mod entity {
    #[derive(Clone)]
    pub struct BaseEntity<ID: Clone> {
        id: ID,
    }

    #[derive(Clone)]
    pub struct AggregateRoot<ID: Clone> {
        base_entity: BaseEntity<ID>,
    }
}

pub mod value_object {
    #[derive(Clone)]
    pub struct BaseId<V: Clone> {
        value: V,
    }

    #[derive(Clone)]
    pub struct CustomerId {
        base_id: BaseId<uuid::Uuid>,
    }

    #[derive(Clone)]
    pub enum OrderApprovalStatus {
        Approved,
        Rejected,
    }

    #[derive(Clone)]
    pub struct OrderId {
        base_id: BaseId<uuid::Uuid>,
    }

    #[derive(Clone)]
    pub enum OrderStatus {
        Pending,
        Approved,
        Cancelling,
        Cancelled,
    }

    #[derive(Clone)]
    pub enum PaymentStatus {
        Completed,
        Failed,
        Canceled,
    }

    #[derive(Clone)]
    pub struct ProductId {
        base_id: BaseId<uuid::Uuid>,
    }

    #[derive(Clone)]
    pub struct RestaurantId {
        base_id: BaseId<uuid::Uuid>,
    }

    pub mod money {

        #[derive(Clone)]
        pub struct Money {
            amount: f64,
        }
        const ZERO: Money = Money { amount: 0.0 };
    }
}

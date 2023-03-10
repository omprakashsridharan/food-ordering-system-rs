pub mod entity {
    use derive_builder::Builder;

    #[derive(Clone, Builder)]
    pub struct BaseEntity<ID: Clone> {
        id: ID,
    }

    #[derive(Clone)]
    pub struct AggregateRoot<ID: Clone> {
        base_entity: BaseEntity<ID>,
    }
}

pub mod value_object {
    use derive_builder::Builder;

    #[derive(Clone, Builder)]
    pub struct BaseId<V: Clone> {
        value: V,
    }

    #[derive(Clone)]
    pub struct CustomerId {
        base_id: BaseId<uuid::Uuid>,
    }

    impl From<uuid::Uuid> for CustomerId {
        fn from(id: uuid::Uuid) -> Self {
            Self {
                base_id: BaseIdBuilder::default().value(id).build().unwrap(),
            }
        }
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

    impl From<uuid::Uuid> for OrderId {
        fn from(id: uuid::Uuid) -> Self {
            Self {
                base_id: BaseIdBuilder::default().value(id).build().unwrap(),
            }
        }
    }

    #[derive(Clone, PartialEq)]
    pub enum OrderStatus {
        Pending,
        Paid,
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

    #[derive(Clone, Builder)]
    pub struct ProductId {
        base_id: BaseId<uuid::Uuid>,
    }

    impl From<uuid::Uuid> for ProductId {
        fn from(id: uuid::Uuid) -> Self {
            Self {
                base_id: BaseIdBuilder::default().value(id).build().unwrap(),
            }
        }
    }

    #[derive(Clone, Builder)]
    pub struct RestaurantId {
        base_id: BaseId<uuid::Uuid>,
    }

    impl From<uuid::Uuid> for RestaurantId {
        fn from(id: uuid::Uuid) -> Self {
            Self {
                base_id: BaseIdBuilder::default().value(id).build().unwrap(),
            }
        }
    }

    pub mod money {
        use std::ops;

        #[derive(Clone)]
        pub struct Money {
            amount: f64,
        }

        impl PartialEq for Money {
            fn eq(&self, other: &Self) -> bool {
                self.amount == other.amount
            }
        }

        impl ops::AddAssign<Money> for Money {
            fn add_assign(&mut self, rhs: Money) {
                self.amount += rhs.amount
            }
        }

        impl ops::Mul<u64> for Money {
            type Output = Money;

            fn mul(self, rhs: u64) -> Self::Output {
                return Money::new(self.amount * (rhs as f64));
            }
        }

        impl Money {
            pub fn new(amount: f64) -> Self {
                Self { amount }
            }
            pub fn is_greater_than_zero(&self) -> bool {
                return self.amount > 0.0;
            }
        }

        pub const ZERO: Money = Money { amount: 0.0 };
    }
}

pub mod event {
    pub mod publisher {
        use super::DomainEvent;

        pub trait DomainEventPublisher<E, T: DomainEvent<E>> {
            fn publish(event: T);
        }
    }

    pub trait DomainEvent<T> {}
}

pub mod error {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum OrderDomainError {
        #[error("the total price of order should be greater than zero")]
        TotalPriceZeroError,
        #[error("the price of order item is invalid")]
        OrderItemPriceInvalid,
        #[error("the price of individual order items does not add up to the total price of order")]
        OrderTotalPriceMismatch,
        #[error("the order status is invalid for {0} operation")]
        InvalidOrderStatus(String),
        #[error("inactive restaurant")]
        InactiveRestaurant,
    }
}

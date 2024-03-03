use crate::{
    ConfirmRequest, FilterRequest, QueryRequest, Reservation, ReservationFilter, ReservationQuery,
    ReserveRequest,
};

macro_rules! impl_new {
    ($name:ident, $field:ident, $type:ty) => {
        impl $name {
            pub fn new(value: $type) -> Self {
                Self {
                    $field: Some(value),
                }
            }
        }
    };
    ($name:ident) => {
        impl $name {
            pub fn new(value: i64) -> Self {
                Self { id: value }
            }
        }
    };
}

impl_new!(ReserveRequest, reservation, Reservation);
impl_new!(FilterRequest, filter, ReservationFilter);
impl_new!(QueryRequest, query, ReservationQuery);
impl_new!(ConfirmRequest);

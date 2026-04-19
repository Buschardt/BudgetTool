pub mod generator;
pub mod journal;
pub mod models;
pub mod periodics;
pub mod prices;
pub mod transactions;

use crate::core::AppState;
use axum::Router;
use axum::routing::{get, put};

pub use periodics::{
    create as create_periodic, delete as delete_periodic, list as list_periodics,
    update as update_periodic,
};
pub use prices::{
    create as create_price, delete as delete_price, list as list_prices, update as update_price,
};
pub use transactions::{
    create as create_transaction, delete as delete_transaction, list as list_transactions,
    update as update_transaction,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/prices", get(list_prices).post(create_price))
        .route("/api/prices/{id}", put(update_price).delete(delete_price))
        .route(
            "/api/transactions",
            get(list_transactions).post(create_transaction),
        )
        .route(
            "/api/transactions/{id}",
            put(update_transaction).delete(delete_transaction),
        )
        .route("/api/periodics", get(list_periodics).post(create_periodic))
        .route(
            "/api/periodics/{id}",
            put(update_periodic).delete(delete_periodic),
        )
}

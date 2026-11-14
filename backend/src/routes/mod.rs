//! HTTP routing.
//!
//! Add a domain by writing `routes/<domain>.rs` exposing a `router()`, then
//! merging it in [`api_router`] — this module stays a thin wiring layer, the
//! way `app/main.py` did.

pub mod access_logs;
pub mod alexa;
pub mod alexa_list_changes;
pub mod delivery_settings;
pub mod dislikes;
pub mod extract;
pub mod google_tasks;
pub mod grocery;
pub mod health;
pub mod intake_settings;
pub mod item_rules;
pub mod order_options;
pub mod order_review;
pub mod products;
pub mod purchases;
pub mod selections;
pub mod spending;
pub mod store_products;
pub mod store_tab;
pub mod triage;
pub mod trolley_handoffs;
pub mod voice;
pub mod voice_list_changes;
pub mod ws;

use axum::Router;

use crate::state::AppState;

/// Every route the backend serves, without the app-wide middleware.
///
/// Takes the state because the store-tab routes carry a layer of their own
/// that reads the stores' configured websites.
pub fn api_router(state: &AppState) -> Router<AppState> {
    // Routes whose writes can change what the list and order pages show:
    // the list, its chosen products and dislike marks, and the order's
    // delivery fees. Item rules are not here: a rule only applies to items
    // added after it.
    let list_changing = Router::new()
        .merge(delivery_settings::router())
        .merge(voice::router())
        .merge(voice_list_changes::router())
        .merge(alexa_list_changes::router())
        .merge(dislikes::router())
        .merge(grocery::router())
        .merge(purchases::router())
        .merge(selections::router())
        .merge(triage::router())
        .merge(trolley_handoffs::router())
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::list_changed::notify_list_changed,
        ));

    Router::new()
        .merge(list_changing)
        .merge(health::router())
        .merge(store_products::router())
        .merge(access_logs::router())
        .merge(alexa::router())
        .merge(google_tasks::router())
        .merge(intake_settings::router())
        .merge(item_rules::router())
        .merge(order_options::router())
        .merge(order_review::router())
        .merge(products::router())
        .merge(purchases::lookup_router())
        .merge(selections::snapshot_router())
        .merge(spending::router())
        .merge(store_tab::router(state.clone()))
        .merge(ws::router())
}

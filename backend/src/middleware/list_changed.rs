//! Tells open browser sessions to read the grocery list again after a write
//! that may have changed it (`docs/features/FEATURE_GROCERY_LIST.md`).
//!
//! Layered on the routers whose writes can change the list, its chosen
//! products or its order (`routes/mod.rs`), so no handler has to remember to
//! publish. The event carries nothing and clients only re-read, so an event
//! for a write that changed nothing is harmless.

use axum::extract::{Request, State};
use axum::http::Method;
use axum::middleware::Next;
use axum::response::Response;

use crate::services::ws_hub::ServerEvent;
use crate::state::AppState;

/// Runs the request, then broadcasts [`ServerEvent::GroceryListChanged`] when
/// it was a write that succeeded.
pub async fn notify_list_changed(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let writes = is_write(request.method());
    let response = next.run(request).await;
    if writes && response.status().is_success() {
        state.hub.broadcast(ServerEvent::GroceryListChanged);
    }
    response
}

/// Whether a request with this method can change anything.
fn is_write(method: &Method) -> bool {
    !matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_and_preflights_are_not_writes() {
        assert!(!is_write(&Method::GET));
        assert!(!is_write(&Method::HEAD));
        assert!(!is_write(&Method::OPTIONS));
    }

    #[test]
    fn every_changing_method_is_a_write() {
        for method in [Method::POST, Method::PUT, Method::PATCH, Method::DELETE] {
            assert!(is_write(&method), "{method}");
        }
    }
}

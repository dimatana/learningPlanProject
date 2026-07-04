// service-common/src/docs.rs

use axum::Router;
use axum::response::Html;
use axum::routing::get;

/// Builds an axum router that exposes `/api-docs/openapi.yaml` and
/// `/swagger-ui`, based on the provided content.
///
/// Each service passes its own `openapi.yaml` and `swagger_ui.html`
/// (included at compile time with `include_str!` in the service's crate),
/// because each has a different contract.
pub fn docs_router(openapi_yaml: &'static str, swagger_html: &'static str) -> Router {
    Router::new()
        .route(
            "/api-docs/openapi.yaml",
            get(move || async move { openapi_yaml }),
        )
        .route(
            "/swagger-ui",
            get(move || async move { Html(swagger_html) }),
        )
}

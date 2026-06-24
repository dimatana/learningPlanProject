use axum::Router;
use axum::routing::get;
use axum::response::Html;
use crate::AppState;

async fn serve_openapi() -> &'static str {
    include_str!("../openapi.yaml")
}

async fn swagger_ui() -> Html<&'static str> {
    Html(include_str!("swagger_ui.html"))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api-docs/openapi.yaml", get(serve_openapi))
        .route("/swagger-ui", get(swagger_ui))
}
use axum::Router;
use axum::response::Html;
use axum::routing::get;

async fn serve_openapi() -> &'static str {
    include_str!("../openapi.yaml")
}

async fn swagger_ui() -> Html<&'static str> {
    Html(include_str!("swagger_ui.html"))
}

pub fn router() -> Router {
    Router::new()
        .route("/api-docs/openapi.yaml", get(serve_openapi))
        .route("/swagger-ui", get(swagger_ui))
}
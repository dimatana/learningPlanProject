use rdkafka::producer::FutureProducer;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub producer: FutureProducer,
}

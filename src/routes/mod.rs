use crate::controllers;
use crate::middlewares;
use crate::models;
use axum::{
    http::HeaderValue,
    middleware,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use std::time::Duration;
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, Any, CorsLayer},
    timeout::TimeoutLayer,
};

pub fn create_route(state: &Arc<models::AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list([
            "https://memelibre.com".parse::<HeaderValue>().unwrap(),
            "http://memelibre.com".parse::<HeaderValue>().unwrap(),
        ]))
        .allow_methods(Any)
        .allow_headers(Any);

    let auth_routes = Router::new()
        .route("/", get(controllers::auth::auth::handler))
        .route("/callback", get(controllers::auth::auth_callback::handler));

    let meme_routes = Router::new()
        .route(
            "/delete/{id}",
            delete(controllers::meme::delete::handler)
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    middlewares::with_is_admin::handler,
                ))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    middlewares::with_auth::handler,
                )),
        )
        .route("/get", get(controllers::meme::get::handler))
        .route("/get/{id}", get(controllers::meme::get_by_id::handler))
        .route("/post", post(controllers::meme::post::handler));

    let likes_routes = Router::new().route(
        "/{meme_id}",
        post(controllers::likes::handler).layer(middleware::from_fn_with_state(
            state.clone(),
            middlewares::with_auth::handler,
        )),
    );

    Router::new()
        .nest(
            "/api",
            Router::new()
                .nest("/auth", auth_routes)
                .nest("/meme", meme_routes)
                .nest("/like", likes_routes)
                .with_state(state.clone()),
        )
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(
            state.config.timeout_duration,
        )))
}

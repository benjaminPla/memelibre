use crate::http_error;
use crate::models;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;

pub async fn handler(
    State(state): State<Arc<models::AppState>>,
    Query(params): Query<models::Pagination>,
) -> Result<Json<Vec<models::Meme>>, (StatusCode, String)> {
    let memes: Vec<models::Meme> = sqlx::query_as(
        "
        SELECT id, image_url, like_count
        FROM memes
        WHERE id < COALESCE($1, 2147483647)
        ORDER BY id DESC
        LIMIT $2
        ",
    )
    .bind(params.offset)
    .bind(&state.config.memes_pull_limit)
    .fetch_all(&state.db)
    .await
    .map_err(|e| http_error!(StatusCode::INTERNAL_SERVER_ERROR, err: e))?;

    Ok(Json(memes))
}

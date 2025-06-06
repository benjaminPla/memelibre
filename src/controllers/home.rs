use axum::{extract::State, response::Html, routing::get, Router};
use serde::Serialize;
use std::env;
use std::sync::Arc;
use tera::Context;

use crate::AppState;

#[derive(Serialize, sqlx::FromRow)]
struct Meme {
    id: i32,
    image_url: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(handler))
}

async fn handler(State(state): State<Arc<AppState>>) -> Html<String> {
    let limit = env::var("MEMES_PULL_LIMIT")
        .expect("Missing MEMES_PULL_LIMIT env var")
        .parse::<i64>()
        .expect("Error parsing MEMES_PULL_LIMIT env var");

    let memes: Vec<Meme> = sqlx::query_as(
        "
        SELECT id, image_url
        FROM memes
        ORDER BY id DESC
        LIMIT $1
        ",
    )
    .bind(limit)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_else(|_| vec![]);

    let mut memes_html = String::new();
    for meme in &memes {
        let mut context = Context::new();
        context.insert("meme", meme);

        let rendered = state
            .tera
            .render("_meme.html", &context)
            .unwrap_or_else(|e| {
                eprintln!("Tera rendering error for meme id {}: {}", meme.id, e);
                "<!-- Error rendering meme -->".to_string()
            });

        memes_html.push_str(&rendered);
    }

    let mut context = Context::new();
    context.insert("memes", &memes_html);

    let rendered = state
        .tera
        .render("home.html", &context)
        .unwrap_or_else(|e| {
            eprintln!("Tera rendering error: {}", e);
            "Template error".to_string()
        });

    Html(rendered)
}

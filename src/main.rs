use std::error::Error;

use askama::Template;
use askama_web::WebTemplate;
use axum::{Router, response::IntoResponse, routing::get, serve};
use tower_http::{compression::CompressionLayer, services::ServeFile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = Router::new()
        .route("/", get(hi))
        .route_service(
            "/massive_chair_angel.jpg",
            ServeFile::new("public/massive_chair_angel.jpg"),
        )
        .layer(CompressionLayer::new().br(true));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    serve(listener, app).await?;

    Ok(())
}

#[derive(Template, WebTemplate)]
#[template(path = "landing.html")]
struct Landing {
    title: String,
}

async fn hi() -> impl IntoResponse {
    Landing {
        title: "caoo".to_string(),
    }
}

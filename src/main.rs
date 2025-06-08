use core::f64;
use std::{error::Error, sync::Arc};

use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    serve::serve,
};
use chrono::Utc;
use rstar::{DefaultParams, RTree};
use serde::Deserialize;
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, services::ServeDir};
use vaktijars::{City, VaktijaColor, VaktijaTime, generate_coord_rtree, prayer_times};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let state = Arc::new(generate_coord_rtree("c1ties500.csv")?);
    let app = Router::new()
        .route("/", get(landing))
        .route("/vaktija", get(vaktija))
        .nest_service("/public", ServeDir::new("public"))
        .with_state(state)
        .layer(CompressionLayer::new().br(true).gzip(true));

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    serve(listener, app).await?;

    Ok(())
}

#[derive(Template, WebTemplate)]
#[template(path = "landing.html")]
struct Landing {
    title: String,
}

async fn landing() -> impl IntoResponse {
    Landing {
        title: "Vaktija.rs".to_string(),
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "vaktija.html")]
struct Vaktija {
    place: String,
    date: String,
    vakat: Vec<VaktijaTime>,
    next_prayer_since_epoch: u64,
    next_prayer_in: u64,
    request_info: VaktijaInfo,
}

#[derive(Debug, Deserialize)]
struct VaktijaInfo {
    latitude: f64,
    longitude: f64,
    timezone: f64,
}

async fn vaktija(
    Query(info): Query<VaktijaInfo>,
    State(rtree): State<Arc<RTree<City, DefaultParams>>>,
) -> Vaktija {
    let mut now = Utc::now().date_naive();
    loop {
        let mut vakat = prayer_times(
            info.latitude,  //.unwrap_or(43.1406),
            info.longitude, // .as_f64().unwrap_or(20.5213),
            info.timezone / 3600.0,
            now,
        );

        let (next_prayer_idx, _) = vakat
            .iter()
            .enumerate()
            .min_by_key(|(_, x)| x.time_remaining() as u64) // crazy time save
            .unwrap();

        // best loop use case ever
        if vakat[next_prayer_idx].time_remaining().is_negative() {
            now = now.succ_opt().unwrap();
            continue;
        }

        vakat[next_prayer_idx].color = VaktijaColor::Active;

        return Vaktija {
            place: rtree
                .nearest_neighbor(&City::new(info.latitude, info.longitude))
                .map_or("😭".to_string(), |x| x.name.clone()),
            date: now.to_string(),
            next_prayer_since_epoch: vakat[next_prayer_idx].since_epoch() as u64,
            next_prayer_in: vakat[next_prayer_idx].time_remaining() as u64,
            vakat,
            request_info: info,
        };
    }
}

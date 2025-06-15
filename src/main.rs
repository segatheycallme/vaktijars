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
use edit_distance::edit_distance;
use rstar::{DefaultParams, RTree};
use serde::Deserialize;
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, services::ServeDir};
use vaktijars::{
    City, VaktijaColor, VaktijaTime, generate_coord_rtree, prayer_times, read_big_cities,
};

struct AppState {
    rtree: RTree<City, DefaultParams>,
    cities: Vec<City>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let rtree = generate_coord_rtree("c1ties500.csv")?;
    println!("RTree generated");
    let cities = read_big_cities("cities15000.csv")?;
    println!("Big cities parsed");
    let app = Router::new()
        .route("/", get(landing))
        .route("/vaktija", get(vaktija))
        .route("/citayyy", get(citayyy))
        .nest_service("/public", ServeDir::new("public"))
        .with_state(Arc::new(AppState { rtree, cities }))
        .layer(CompressionLayer::new().br(true).gzip(true));

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("Listening on 0.0.0.0:3000");
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
    precise: bool,
}

#[derive(Debug, Deserialize)]
struct VaktijaInfo {
    latitude: f64,
    longitude: f64,
    timezone: f64,
    precise: Option<String>,
    offset: Option<String>,
}

async fn vaktija(Query(info): Query<VaktijaInfo>, State(state): State<Arc<AppState>>) -> Vaktija {
    let mut now = Utc::now().date_naive();
    let precise = match info.precise {
        Some(x) => &x == "on",
        None => false,
    };
    let offset = match info.offset {
        Some(x) => &x == "on",
        None => false,
    };
    loop {
        let mut vakat = prayer_times(
            info.latitude,  //.unwrap_or(43.1406),
            info.longitude, // .as_f64().unwrap_or(20.5213),
            info.timezone / 3600.0,
            now,
            offset,
        );

        let (next_prayer_idx, _) = vakat
            .iter()
            .enumerate()
            .min_by_key(|(_, x)| (x.time_remaining() - 1) as u64) // crazy time save
            .unwrap();

        // best loop use case ever
        if vakat[next_prayer_idx].time_remaining().is_negative() {
            now = now.succ_opt().unwrap();
            continue;
        }

        vakat[next_prayer_idx].color = VaktijaColor::Active;

        return Vaktija {
            place: state
                .rtree
                .nearest_neighbor(&City::new(info.latitude, info.longitude))
                .map_or("😭".to_string(), |x| x.name.clone()),
            date: now.to_string(),
            next_prayer_since_epoch: vakat[next_prayer_idx].since_epoch() as u64,
            vakat,
            precise,
        };
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "active_search.html")]
struct ActiveSearch {
    results: Vec<(isize, City)>,
    lat: f64,
    lon: f64,
}

#[derive(Deserialize)]
struct CitySearch {
    q: Option<String>,
}

async fn citayyy(
    Query(query): Query<CitySearch>,
    State(state): State<Arc<AppState>>,
) -> ActiveSearch {
    let search_query = query.q.unwrap().to_lowercase(); // mucno mi stv
    let mut closest_match: Vec<_> = state
        .cities
        .iter()
        .map(|a| {
            (
                edit_distance(&a.lower, &search_query) as isize
                    - (a.lower.starts_with(&search_query) as isize * 10),
                a,
            )
        })
        .collect();
    closest_match.sort_unstable_by_key(|x| x.0);
    let fantastiche_funf = dbg!(closest_match.split_at(5).0);
    ActiveSearch {
        lat: fantastiche_funf[0].1.lat,
        lon: fantastiche_funf[0].1.lon,
        results: fantastiche_funf
            .iter()
            .map(|a| (a.0, a.1.clone()))
            .collect(),
    }
}

use core::f64;
use std::{
    error::Error,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Router,
    extract::{ConnectInfo, State, connect_info::Connected},
    response::IntoResponse,
    routing::get,
    serve::{IncomingStream, serve},
};
use chrono::{Datelike, NaiveDate, Utc};
use julian::Calendar;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, services::ServeFile};
use vaktijars::astronomical_measures;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    prayer_times(43.1406976, 20.5213617);
    todo!();
    let state = Arc::new(Client::builder().brotli(true).build()?);
    let app = Router::new()
        .route("/", get(landing))
        .route("/vaktija", get(vaktija))
        .route_service(
            "/massive_chair_angel.jpg",
            ServeFile::new("public/massive_chair_angel.jpg"),
        )
        .with_state(state)
        .layer(CompressionLayer::new().br(true));

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    serve(
        listener,
        app.into_make_service_with_connect_info::<IpInfo>(),
    )
    .await?;

    Ok(())
}

#[derive(Debug, Clone)]
struct IpInfo {
    ip: String,
}

impl Connected<IncomingStream<'_, TcpListener>> for IpInfo {
    fn connect_info(target: IncomingStream<'_, TcpListener>) -> Self {
        IpInfo {
            ip: target.remote_addr().ip().to_string(),
        }
    }
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
}

struct VaktijaTime {
    name: String,
    absolute_time: String,
    relative_time: String,
}

#[derive(Debug, Deserialize)]
struct IpWhoIs {
    timezone: f64,
    latitude: f64,
    longitude: f64,
}

async fn vaktija(
    ConnectInfo(info): ConnectInfo<IpInfo>,
    State(client): State<Arc<Client>>,
) -> Vaktija {
    let mut json: Value = serde_json::from_str(
        &client
            .get(format!(
                "https://ipwho.is/{}?fields=latitude,longitude,timezone.offset",
                info.ip
            ))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
    )
    .unwrap();

    json["timezone"] = json["timezone"]["offset"].clone();

    let user_info: IpWhoIs = dbg!(serde_json::from_value(json).unwrap());

    let vakat = prayer_times(user_info.latitude, user_info.longitude);
    //         name: "Zora".to_string(),
    //         name: "Izlazak Sunca".to_string(),
    //         name: "Podne".to_string(),
    //         name: "Ikindija".to_string(),
    //         name: "Akšam".to_string(),
    //         name: "Jacija".to_string(),

    Vaktija {
        place: "Novi Pazar".to_string(),
        date: "sri, 4. juni 2025 / 8. zu-l-hidždže 1446".to_string(),
        vakat,
    }
}

fn prayer_times(lat: f64, lon: f64) -> Vec<VaktijaTime> {
    let (solar_declination, equation_of_time) = astronomical_measures();
    let solar_noon = 12.0 + 2.0 - lon / 15.0 - equation_of_time;
    dbg!(solar_noon);
    let up = (-0.833_f64).to_radians().sin()
        + lat.to_radians().sin() * solar_declination.to_radians().sin();
    let down = lat.to_radians().cos() * solar_declination.to_radians().cos();
    let izlazak = solar_noon - (dbg!((dbg!(-up) / dbg!(down))).acos() / 15.0_f64.to_radians());
    let zalazak = solar_noon + (dbg!((dbg!(-up) / dbg!(down))).acos() / 15.0_f64.to_radians());
    dbg!(izlazak);
    dbg!(zalazak);
    todo!();
    vec![]
}

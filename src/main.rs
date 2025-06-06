use core::f64;
use std::{error::Error, sync::Arc};

use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Router,
    extract::{ConnectInfo, State, connect_info::Connected},
    response::IntoResponse,
    routing::get,
    serve::{IncomingStream, serve},
};
use reqwest::Client;
use serde_json::Value;
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, services::ServeFile};
use vaktijars::astronomical_measures;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // prayer_times(45.75, 16.0, 2.0);
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
    hours: f64,
}

impl VaktijaTime {
    fn new(name: &str, mut hours: f64) -> Self {
        if hours < 0.0 {
            hours += 24.0;
        }
        if hours > 24.0 {
            hours -= 24.0;
        }

        VaktijaTime {
            name: name.to_string(),
            hours,
        }
    }
    fn absolute_time(&self) -> &String {
        &self.name
    }
    fn relative_time(&self) -> &String {
        &self.name
    }
}

async fn vaktija(
    ConnectInfo(info): ConnectInfo<IpInfo>,
    State(client): State<Arc<Client>>,
) -> Vaktija {
    let json: Value = serde_json::from_str(
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

    let vakat = prayer_times(
        json["latitude"].as_f64().unwrap(),
        json["longitude"].as_f64().unwrap(),
        json["timezone"]["offset"].as_f64().unwrap() / 3600.0,
    );

    Vaktija {
        place: "Novi Pazar".to_string(),
        date: "sri, 4. juni 2025 / 8. zu-l-hidždže 1446".to_string(),
        vakat,
    }
}

// praytimes.org/calculations
fn prayer_times(lat: f64, lon: f64, timezone: f64) -> Vec<VaktijaTime> {
    let (solar_declination, equation_of_time) = astronomical_measures();
    let solar_noon = 12.0 + timezone - lon / 15.0 - equation_of_time;
    let t = |a: f64| {
        (-(a.to_radians().sin() + lat.to_radians().sin() * solar_declination.to_radians().sin())
            / (lat.to_radians().cos() * solar_declination.to_radians().cos()))
        .acos()
            / 15.0_f64.to_radians()
    };
    let a = |t: f64| {
        ((((1.0 / ((lat - solar_declination).to_radians().tan() + t))
            .atan()
            .sin())
            - lat.to_radians().sin() * solar_declination.to_radians().sin())
            / (lat.to_radians().cos() * solar_declination.to_radians().cos()))
        .acos()
            / 15.0_f64.to_radians()
    };
    let sunrise = solar_noon - t(0.833);
    let sunset = solar_noon + t(0.833);
    let fajr = solar_noon - t(18.0); // muslim world league angles
    let isha = solar_noon + t(17.0); // muslim world league angles
    let asr = solar_noon + a(1.0);
    vec![
        VaktijaTime::new("Zora", fajr),
        VaktijaTime::new("Izlazak Sunca", sunrise),
        VaktijaTime::new("Podne", solar_noon),
        VaktijaTime::new("Ikindija", asr),
        VaktijaTime::new("Akšam", sunset),
        VaktijaTime::new("Jacija", isha),
    ]
}

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
use serde::Deserialize;
use serde_json::Value;
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, services::ServeFile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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

    let jd = 0.0;
    let d = jd - 2451545.0; // jd is the given Julian date 

    let g = 357.529 + 0.98560028 * d;
    let q = 280.459 + 0.98564736 * d;
    let L = q + 1.915 * f64::sin(g) + 0.020 * f64::sin(2.0 * g);

    let R = 1.00014 - 0.01671 * f64::cos(g) - 0.00014 * f64::cos(2.0 * g);
    let e = 23.439 - 0.00000036 * d;
    let RA = f64::atan2(f64::cos(e) * f64::sin(L), f64::cos(L)) / 15.0;

    let D = f64::asin(f64::sin(e) * f64::sin(L)); // declination of the Sun
    let EqT = q / 15.0 - RA; // equation of time

    let vakat = vec![
        VaktijaTime {
            name: "Zora".to_string(),
            absolute_time: "02:47".to_string(),
            relative_time: "prije 17 sati".to_string(),
        },
        VaktijaTime {
            name: "Izlazak Sunca".to_string(),
            absolute_time: "02:47".to_string(),
            relative_time: "prije 17 sati".to_string(),
        },
        VaktijaTime {
            name: "Podne".to_string(),
            absolute_time: "02:47".to_string(),
            relative_time: "prije 17 sati".to_string(),
        },
        VaktijaTime {
            name: "Ikindija".to_string(),
            absolute_time: "02:47".to_string(),
            relative_time: "prije 17 sati".to_string(),
        },
        VaktijaTime {
            name: "Akšam".to_string(),
            absolute_time: "02:47".to_string(),
            relative_time: "prije 17 sati".to_string(),
        },
        VaktijaTime {
            name: "Jacija".to_string(),
            absolute_time: "02:47".to_string(),
            relative_time: "prije 17 sati".to_string(),
        },
    ];

    Vaktija {
        place: "Novi Pazar".to_string(),
        date: "sri, 4. juni 2025 / 8. zu-l-hidždže 1446".to_string(),
        vakat,
    }
}

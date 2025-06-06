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
use chrono::{FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};
use reqwest::Client;
use serde_json::Value;
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, services::ServeFile};
use vaktijars::astronomical_measures;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // prayer_times(59.32982, 18.086, 2.0, Utc::now().date_naive());
    // todo!();
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
    next_prayer: u32,
}

#[derive(Debug)]
enum VaktijaColor {
    Base,
    Active,
}

#[derive(Debug)]
struct VaktijaTime {
    name: String,
    date_time: Option<NaiveDateTime>,
    now: NaiveDateTime,
    color: VaktijaColor,
}

impl VaktijaTime {
    fn new(name: &str, hours: f64, offset: FixedOffset) -> Self {
        // this line assumes that the day hasn't changed since calculating prayer times
        let today = Utc::now().date_naive();

        let date_time = match hours {
            ..0.0 => Some(
                today.pred_opt().unwrap().and_time(
                    NaiveTime::from_num_seconds_from_midnight_opt(
                        ((hours + 24.0) * 3600.0) as u32,
                        0,
                    )
                    .unwrap(),
                ),
            ),
            0.0..=24.0 => Some(
                today.and_time(
                    NaiveTime::from_num_seconds_from_midnight_opt(
                        ((hours + 0.0) * 3600.0) as u32,
                        0,
                    )
                    .unwrap(),
                ),
            ),
            24.0.. => Some(
                today.succ_opt().unwrap().and_time(
                    NaiveTime::from_num_seconds_from_midnight_opt(
                        ((hours - 24.0) * 3600.0) as u32,
                        0,
                    )
                    .unwrap(),
                ),
            ),
            x if x.is_nan() => None,
            x => panic!("????, magic time: {x}"),
        };

        VaktijaTime {
            name: name.to_string(),
            date_time,
            now: Utc::now().naive_utc().checked_add_offset(offset).unwrap(),
            color: VaktijaColor::Base,
        }
    }
    fn absolute_time(&self) -> String {
        if let Some(date_time) = self.date_time {
            return format!(
                "{:0>2}:{:0>2}",
                date_time.num_seconds_from_midnight() / 3600,
                (date_time.num_seconds_from_midnight() + 1) / 60 % 60
            );
        }
        "N/A".to_string()
    }
    fn relative_time(&self) -> String {
        if let Some(date_time) = self.date_time {
            let mut since = date_time.signed_duration_since(self.now);

            // pakao
            let before_or_after = if since.num_seconds() > 0 {
                "za"
            } else {
                since = since.checked_mul(-1).unwrap();
                "pre"
            };
            let (number, unit) = match since.num_seconds() {
                3600.. => (
                    since.num_hours(),
                    match since.num_hours() % 20 {
                        1 => "sat",
                        2..=5 => "sata",
                        _ => "sati",
                    },
                ),
                60..3600 => (
                    since.num_minutes(),
                    match since.num_minutes() % 20 {
                        1 => "minut",
                        _ => "minuta",
                    },
                ),
                0..60 => (
                    since.num_seconds(),
                    match since.num_seconds() % 20 {
                        1 => "sekunda",
                        2..=5 => "sekunde",
                        _ => "sekundi",
                    },
                ),
                _ => panic!("????, negative since seconds"),
            };
            return format!("{} {} {}", before_or_after, number, unit);
        }
        "N/A".to_string()
    }
    fn time_remaining(&self) -> i32 {
        self.date_time.map_or(-1, |date_time| {
            date_time.signed_duration_since(self.now).num_seconds() as i32
        })
    }
    fn get_color(&self) -> String {
        match self.color {
            VaktijaColor::Base => "stone-800",
            VaktijaColor::Active => "stone-400",
        }
        .to_string()
    }
}

async fn vaktija(
    ConnectInfo(info): ConnectInfo<IpInfo>,
    State(client): State<Arc<Client>>,
) -> Vaktija {
    let json: Value = dbg!(
        serde_json::from_str(
            &client
                .get(format!(
                    "https://ipwho.is/{}?fields=latitude,longitude,timezone.offset,city",
                    info.ip
                ))
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap(),
        )
        .unwrap()
    );

    let now = Utc::now().date_naive();
    let mut vakat = prayer_times(
        json["latitude"].as_f64().unwrap(),
        json["longitude"].as_f64().unwrap(),
        json["timezone"]["offset"].as_f64().unwrap() / 3600.0,
        now,
    );

    let (next_prayer_idx, _) = vakat
        .iter()
        .enumerate()
        .min_by_key(|(_, x)| x.time_remaining() as u32) // crazy time save
        .unwrap();
    vakat[next_prayer_idx].color = VaktijaColor::Active;

    Vaktija {
        place: json["city"].as_str().unwrap().to_string(),
        date: now.to_string(),
        next_prayer: vakat[next_prayer_idx].time_remaining() as u32,
        vakat,
    }
}

// praytimes.org/calculations
fn prayer_times(lat: f64, lon: f64, timezone: f64, now: NaiveDate) -> Vec<VaktijaTime> {
    let (solar_declination, equation_of_time) = astronomical_measures(now);
    let solar_noon = 12.0 + timezone - lon / 15.0 - equation_of_time;
    let t = |a: f64| {
        let up = -a.to_radians().sin() - lat.to_radians().sin() * solar_declination.sin();
        let down = lat.to_radians().cos() * solar_declination.cos();
        (up / down).acos() / 15.0_f64.to_radians()
    };
    let a = |t: f64| {
        ((((1.0 / ((lat.to_radians() - solar_declination).tan() + t))
            .atan()
            .sin())
            - lat.to_radians().sin() * solar_declination.sin())
            / (lat.to_radians().cos() * solar_declination.cos()))
        .acos()
            / 15.0_f64.to_radians()
    };
    let sunrise = solar_noon - t(0.833);
    let sunset = solar_noon + t(0.833);
    let fajr = solar_noon - t(18.0); // muslim world league angles
    let isha = solar_noon + t(17.0); // muslim world league angles
    let asr = solar_noon + a(1.0);

    let offset = FixedOffset::east_opt((timezone * 3600.0) as i32).unwrap();
    vec![
        VaktijaTime::new("Zora", fajr, offset),
        VaktijaTime::new("Izlazak Sunca", sunrise, offset),
        VaktijaTime::new("Podne", solar_noon, offset),
        VaktijaTime::new("Ikindija", asr, offset),
        VaktijaTime::new("Akšam", sunset, offset),
        VaktijaTime::new("Jacija", isha, offset),
    ]
}

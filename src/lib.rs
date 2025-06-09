use std::error::Error;

use chrono::Datelike;
use chrono::FixedOffset;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::Timelike;
use chrono::Utc;
use rstar::DefaultParams;
use rstar::Point;
use rstar::RTree;

#[derive(Debug)]
pub enum VaktijaColor {
    Base,
    Active,
}

#[derive(Debug)]
pub struct VaktijaTime {
    pub name: String,
    pub date_time: Option<NaiveDateTime>,
    pub now: NaiveDateTime,
    pub color: VaktijaColor,
}

impl VaktijaTime {
    fn new(name: &str, hours: f64, today: NaiveDate, offset: FixedOffset) -> Self {
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
    pub fn absolute_time(&self, secs: &bool) -> String {
        if let Some(date_time) = self.date_time {
            return if *secs {
                format!(
                    "{:0>2}:{:0>2}:{:0>2}",
                    date_time.num_seconds_from_midnight() / 3600,
                    date_time.num_seconds_from_midnight() / 60 % 60,
                    date_time.num_seconds_from_midnight() % 60
                )
            } else {
                format!(
                    "{:0>2}:{:0>2}",
                    date_time.num_seconds_from_midnight() / 3600,
                    date_time.num_seconds_from_midnight() / 60 % 60
                )
            };
        }
        "N/A".to_string()
    }
    pub fn time_remaining(&self) -> i64 {
        self.date_time.map_or(-1, |date_time| {
            date_time.signed_duration_since(self.now).num_seconds()
        })
    }
    pub fn since_epoch(&self) -> i64 {
        self.date_time
            .map_or(-1, |date_time| date_time.and_utc().timestamp())
    }
    pub fn get_color(&self) -> String {
        match self.color {
            VaktijaColor::Base => "stone-800",
            VaktijaColor::Active => "stone-400",
        }
        .to_string()
    }
}

// calendrical calculations
pub fn astronomical_measures(now: NaiveDate) -> (f64, f64) {
    let jd: julian::Date = now.into();
    let c = (jd.julian_day_number() as f64 - 2_451_545.0) / 36525.0;

    // degrees land
    let half_life = 280.46645 + 36000.76983 * c + 0.0003032 * c * c; // L0
    let anomaly = 357.52910 + 35999.05030 * c - 0.0001559 * c * c - 0.00000048 * c * c * c; // M
    let eccentricity = 0.016708617 - 0.000042037 * c - 0.0000001236 * c; // e
    let obliquity = (23.0 + 26.0 / 60.0 + 21.448 / 3600.0)
        + (-(46.8150 / 3600.0) * c - (0.00059 / 3600.0) * c * c + (0.001813) * c * c * c); // weird e

    // radians land
    let y = (obliquity.to_radians() / 2.0).tan() * ((obliquity).to_radians() / 2.0).tan();
    let eot = y * (2.0 * half_life.to_radians()).sin()
        - 2.0 * eccentricity * anomaly.to_radians().sin()
        + 4.0
            * eccentricity
            * y
            * anomaly.to_radians().sin()
            * (2.0 * half_life.to_radians()).cos()
        - 0.5 * y * y * (4.0 * half_life.to_radians()).sin()
        - 1.25 * eccentricity * eccentricity * (2.0 * anomaly).to_radians().sin();

    let eot_hours = eot * (180.0 / core::f64::consts::PI) / 15.0;

    let jan1 = NaiveDate::from_ymd_opt(now.year(), 1, 1).unwrap();
    let days = (now - jan1).num_days() as f64;

    // approx declination
    let d = (-23.45 * ((360.0 / 365.0) * (days + 10.0)).to_radians().cos()).to_radians();

    (d, eot_hours)
}

// praytimes.org/calculations
pub fn prayer_times(
    lat: f64,
    lon: f64,
    timezone: f64,
    now: NaiveDate,
    safety: bool,
) -> Vec<VaktijaTime> {
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

    let safety_minutes = if safety {
        [-10.0, -12.0, 2.0, 6.0, 12.0, 2.0]
    } else {
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    };

    let offset = FixedOffset::east_opt((timezone * 3600.0) as i32).unwrap();
    vec![
        VaktijaTime::new("Zora", fajr + safety_minutes[0] / 60.0, now, offset),
        VaktijaTime::new(
            "Izlazak Sunca",
            sunrise + safety_minutes[1] / 60.0,
            now,
            offset,
        ),
        VaktijaTime::new("Podne", solar_noon + safety_minutes[2] / 60.0, now, offset),
        VaktijaTime::new("Ikindija", asr + safety_minutes[3] / 60.0, now, offset),
        VaktijaTime::new("Akšam", sunset + safety_minutes[4] / 60.0, now, offset),
        VaktijaTime::new("Jacija", isha + safety_minutes[5] / 60.0, now, offset),
    ]
}

#[derive(Clone, Debug, PartialEq)]
pub struct City {
    lat: f64,
    lon: f64,
    pub name: String,
}

impl Point for City {
    type Scalar = f64;
    const DIMENSIONS: usize = 2;

    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        City {
            lat: generator(0),
            lon: generator(1),
            name: ":3".to_string(),
        }
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.lat,
            1 => self.lon,
            _ => unreachable!(),
        }
    }
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.lat,
            1 => &mut self.lon,
            _ => unreachable!(),
        }
    }
}

impl City {
    pub fn new(lat: f64, lon: f64) -> Self {
        City {
            lat,
            lon,
            name: String::from("Novi Pazar"),
        }
    }
}

pub fn generate_coord_rtree(
    path: &'static str,
) -> Result<RTree<City, DefaultParams>, Box<dyn Error>> {
    let cities: Vec<City> = csv::Reader::from_path(path)?
        .records()
        .filter_map(|record| {
            let Ok(record) = record else {
                eprintln!("error parsing csv: {}", record.unwrap_err());
                return None;
            };
            Some(City {
                lat: record.get(1).unwrap().parse().unwrap(),
                lon: record.get(2).unwrap().parse().unwrap(),
                name: record.get(0).unwrap().to_owned(),
            })
        })
        .collect();
    Ok(RTree::bulk_load(cities))
}

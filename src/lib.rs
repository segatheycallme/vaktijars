use chrono::Datelike;
use chrono::NaiveDate;
use chrono::Utc;

pub fn astronomical_measures() -> (f64, f64) {
    let now = Utc::now().date_naive();
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

    let eot_minutes = eot * (180.0 / core::f64::consts::PI) / 15.0;

    let jan1 = NaiveDate::from_ymd_opt(now.year(), 1, 1).unwrap();
    let days = (now - jan1).num_days() as f64;

    let d = -23.45 * ((360.0 / 365.0) * days + 10.0).to_radians().cos();

    dbg!((d, eot_minutes))
}

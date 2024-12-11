use actix_web::cookie::time;
use chrono::{DateTime, Utc};
use std::f64::consts::PI;
use time::OffsetDateTime;

pub fn extract_timestamp(time: Option<gpx::Time>) -> i64 {
    match time {
        Some(t) => {
            let offset_datetime: OffsetDateTime = t.into();

            let chrono_time: DateTime<Utc> = DateTime::from_timestamp(
                offset_datetime.unix_timestamp(),
                offset_datetime.nanosecond(),
            )
            .expect("Failed to convert OffsetDateTime to DateTime<Utc>");

            chrono_time.timestamp()
        },
        None => 0,
    }
}

pub fn calculate_distance(lat: f64, lon: f64, next_lat: f64, next_lon: f64) -> f64 {
    let r = 6371_000.0;
    let to_rad = |deg: f64| deg * PI / 180.0;

    let dlat = to_rad(next_lat - lat);
    let dlon = to_rad(next_lon - lon);
    let a = (dlat / 2.0).sin().powi(2)
        + lat.to_radians().cos() * next_lat.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    r * c
}

// pub fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
//     let r = 6371_000.0;  // Радиус Земли в метрах
//     let phi1 = lat1.to_radians();
//     let phi2 = lat2.to_radians();
//     let delta_phi = (lat2 - lat1).to_radians();
//     let delta_lambda = (lon2 - lon1).to_radians();

//     let a = (delta_phi / 2.0).sin() * (delta_phi / 2.0).sin()
//         + phi1.cos() * phi2.cos() * (delta_lambda / 2.0).sin() * (delta_lambda / 2.0).sin();
//     let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

//     r * c
// }

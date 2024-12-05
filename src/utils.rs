use chrono::{DateTime, Utc};
use time::OffsetDateTime;
use std::f64::consts::PI;

pub fn extract_timestamp(time: Option<gpx::Time>) -> i64 {
    match time {
        Some(t) => {
            let offset_datetime: OffsetDateTime = t.into(); 

         
            let chrono_time: DateTime<Utc> = DateTime::from_timestamp(offset_datetime.unix_timestamp(),
             offset_datetime.nanosecond())
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


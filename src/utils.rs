use chrono::{DateTime, Utc};
use time::OffsetDateTime;
use std::f64::consts::PI;

pub fn extract_timestamp(time: Option<gpx::Time>) -> i64 {
    match time {
        Some(t) => {
            let offset_datetime: OffsetDateTime = t.into(); // Преобразуем в OffsetDateTime

            // Используем unwrap() для извлечения значения из Option
            let chrono_time: DateTime<Utc> = DateTime::from_timestamp(offset_datetime.unix_timestamp(),
             offset_datetime.nanosecond())
                .expect("Failed to convert OffsetDateTime to DateTime<Utc>");

            chrono_time.timestamp() // Получаем timestamp
        },
        None => 0, // Если времени нет, возвращаем 0
    }
}

// Вычисление расстояния между координатами
pub fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371_000.0; // Радиус Земли в метрах
    let to_rad = |deg: f64| deg * PI / 180.0;

    let dlat = to_rad(lat2 - lat1);
    let dlon = to_rad(lon2 - lon1);
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    r * c // Расстояние в метрах
}


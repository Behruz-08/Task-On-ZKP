// use chrono::{Duration, Utc};
// use std::error::Error;
// use std::str::FromStr;
// use winterfell::{ ProofOptions, TraceTable};
// // Основная структура для Waypoint
// #[derive(Debug)]
// pub struct Waypoint {
//     pub lat: f64,
//     pub lon: f64,
//     pub timestamp: Option<chrono::DateTime<Utc>>,
// }

// impl Waypoint {
//     pub fn new(lat: f64, lon: f64, timestamp: Option<chrono::DateTime<Utc>>) -> Self {
//         Self { lat, lon, timestamp }
//     }
// }

// // Парсинг GPX файла
// pub fn parse_gpx(gpx_data: &str) -> Result<Vec<Waypoint>, Box<dyn Error>> {
//     let mut waypoints = Vec::new();

//     let document = roxmltree::Document::from(gpx_data)?;
//     for node in document.descendants().filter(|n| n.has_tag_name("trkpt")) {
//         let lat = node.attribute("lat").unwrap_or("0").parse::<f64>()?;
//         let lon = node.attribute("lon").unwrap_or("0").parse::<f64>()?;
//         let timestamp = node
//             .descendants()
//             .find(|n| n.has_tag_name("time"))
//             .and_then(|n| n.text())
//             .map(|t| chrono::DateTime::parse_from_rfc3339(t).ok().map(|dt| dt.with_timezone(&Utc)))
//             .flatten();

//         waypoints.push(Waypoint::new(lat, lon, timestamp));
//     }

//     Ok(waypoints)
// }

// // Временное распределение точек
// pub fn distribute_time(
//     waypoints: &mut Vec<Waypoint>,
//     start_time: chrono::DateTime<Utc>,
//     end_time: chrono::DateTime<Utc>,
// ) {
//     let total_points = waypoints.len();
//     if total_points == 0 {
//         return;
//     }

//     let interval = (end_time - start_time).num_seconds() as usize / total_points;

//     for (i, waypoint) in waypoints.iter_mut().enumerate() {
//         waypoint.timestamp = Some(start_time + Duration::seconds((i * interval) as i64));
//     }
// }

// // Пример структуры публичных входных данных
// #[derive(Debug, PartialEq)]
// pub struct PublicInputs {
//     pub lat: f64,
//     pub lon: f64,
//     pub next_lat: f64,
//     pub next_lon: f64,
// }

// // Пример провайдера доказательств
// pub struct GpsProver {
//     options: ProofOptions,
// }

// impl GpsProver {
//     pub fn new(options: ProofOptions) -> Self {
//         Self { options }
//     }

//     pub fn get_pub_inputs(&self, trace: &TraceTable<B>) -> PublicInputs {
//         PublicInputs {
//             lat: trace.get(0, 0).to_f64().unwrap_or_default(),
//             lon: trace.get(1, 0).to_f64().unwrap_or_default(),
//             next_lat: trace.get(0, trace.length() - 1).to_f64().unwrap_or_default(),
//             next_lon: trace.get(1, trace.length() - 1).to_f64().unwrap_or_default(),
//         }
//     }

//     pub fn prove(&self, trace: TraceTable) -> Result<(), &'static str> {
//         let _ = trace;
//         // Здесь добавьте реализацию генерации доказательства
//         Ok(())
//     }
// }

// // Проверка доказательства
// pub fn verify_gps_trip(
//     _lat: f64,
//     _lon: f64,
//     _next_lat: f64,
//     _next_lon: f64,
//     _proof: (),
// ) -> Result<(), &'static str> {
//     // Реализация проверки
//     Ok(())
// }

// #[cfg(test)]
// mod tests {
//     use prover::FieldExtension;

//     use super::*;

//     #[test]
//     fn test_empty_gpx_file() {
//         let result = parse_gpx("");
//         assert!(result.is_err(), "Ожидается ошибка для пустого файла");
//     }

//     #[test]
//     fn test_gpx_file_without_points() {
//         let gpx_data = r#"<gpx><trk><trkseg></trkseg></trk></gpx>"#;
//         let result = parse_gpx(gpx_data);
//         assert!(result.unwrap().is_empty(), "Ожидались пустые точки трека");
//     }

//     #[test]
//     fn test_valid_gpx_file() {
//         let gpx_data = r#"
//             <gpx>
//                 <trk>
//                     <trkseg>
//                         <trkpt lat="37.7749" lon="-122.4194">
//                             <time>2024-12-12T10:00:14Z</time>
//                         </trkpt>
//                     </trkseg>
//                 </trk>
//             </gpx>
//         "#;
//         let result = parse_gpx(gpx_data).unwrap();
//         assert_eq!(result.len(), 1);
//         assert_eq!(result[0].lat, 37.7749);
//         assert_eq!(result[0].lon, -122.4194);
//     }

//     #[test]
//     fn test_time_distribution() {
//         let start_time = Utc::now();
//         let end_time = start_time + Duration::hours(1);
//         let mut points = vec![
//             Waypoint::new(37.7749, -122.4194, None),
//             Waypoint::new(37.7750, -122.4195, None),
//             Waypoint::new(37.7751, -122.4196, None),
//         ];

//         distribute_time(&mut points, start_time, end_time);

//         for (i, waypoint) in points.iter().enumerate() {
//             assert!(
//                 waypoint.timestamp.unwrap() >= start_time && waypoint.timestamp.unwrap() <= end_time,
//                 "Время должно быть в пределах заданного интервала"
//             );
//             if i > 0 {
//                 assert!(
//                     waypoint.timestamp.unwrap() > points[i - 1].timestamp.unwrap(),
//                     "Каждая следующая точка должна иметь большее время"
//                 );
//             }
//         }
//     }

//     #[test]
//     fn test_public_inputs_generation() {
//         let trace = TraceTable::new(6, 10);
//         let options = ProofOptions::new(
//             32, 16, 0, FieldExtension::None, 127, 8
//         );
      
//         let prover = GpsProver::new(options);
//         let public_inputs = prover.get_pub_inputs(&trace);
//         FieldExtension::None,
//         assert_eq!(public_inputs.lat, trace.get(0, 0).to_f64().unwrap_or_default());
//         assert_eq!(public_inputs.next_lon, trace.get(1, trace.length() - 1).to_f64().unwrap_or_default());
//     }

//     #[test]
//     fn test_proof_verification() {
//         let trace = TraceTable::new(6, 10);
//         let options = ProofOptions::new(
//             32, 16, 0, FieldExtension::None, 127, 8
//         );
//         let prover = GpsProver::new(options);
//         if let Ok(proof) = prover.get_pub_inputs(&trace) {
//             assert!(verify_gps_trip(1.0, 2.0, 3.0, 4.0, proof).is_ok(), "Доказательство должно быть верным");
//         } else {
//             panic!("Доказательство не сгенерировано");
//         }
//     }
// }

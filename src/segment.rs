
use quick_xml::{Reader, events::Event};
use quick_xml::name::QName;
use serde::Deserialize;
pub struct SegmentConfig {
    pub segment_length: u64, 
}

#[derive(Deserialize, Debug, Clone)]
pub struct GpsPoint {
    pub lat: f64,
    pub lon: f64,
    pub _time: u64,
}

#[derive(Debug, Deserialize)]
pub struct Waypoint {
    #[serde(rename = "lat")]
    pub lat: f64,
    #[serde(rename = "lon")]
    pub lon: f64,
    #[serde(rename = "ele")]
    pub _ele: f64,
    #[serde(rename = "time")]
    pub time: String,
}



pub fn parse_gpx(gpx_data: &str) -> Result<Vec<GpsPoint>, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_str(gpx_data);
    reader.config_mut().trim_text(true);
    let mut waypoints: Vec<Waypoint> = Vec::new();
    let mut current_time: Option<String> = None;
    let mut lat: Option<f64> = None;
    let mut lon: Option<f64> = None;

    while let Ok(event) = reader.read_event() {
        match event {
            Event::Start(ref e) if e.name() == QName(b"wpt") => {
                lat = e.attributes()
                    .find(|attr| attr.as_ref().unwrap().key == QName(b"lat"))
                    .map(|attr| attr.unwrap().value)
                    .and_then(|value| String::from_utf8_lossy(&value).parse().ok());

                lon = e.attributes()
                    .find(|attr| attr.as_ref().unwrap().key == QName(b"lon"))
                    .map(|attr| attr.unwrap().value)
                    .and_then(|value| String::from_utf8_lossy(&value).parse().ok());

                if lat.is_some() && lon.is_some() {
                    current_time = None;
                }
            }
            Event::Start(ref e) if e.name() == QName(b"time") => {
                current_time = Some(String::new());
            }
            Event::Text(e) => {
                if let Some(ref mut time) = current_time {
                    time.push_str(&String::from_utf8_lossy(&e));
                }
            }
            Event::End(ref e) if e.name() == QName(b"time") => {
                if let Some(time) = current_time.take() {
                    if let (Some(lat), Some(lon)) = (lat, lon) {
                        let waypoint = Waypoint {
                            lat,
                            lon,
                            _ele: 0.0,
                            time,
                        };
                        waypoints.push(waypoint);
                    }
                }
            }
            Event::Eof => { break; }
           
            _ => {}
        }
    }

    let gps_points = waypoints
        .into_iter()
        .map(|wp| GpsPoint {
            lat: wp.lat,
            lon: wp.lon,
            _time: wp.time.parse().unwrap_or_default(),
        })
        .collect();

    Ok(gps_points)
}

// pub fn split_gps_into_segments(
//     gps_points: Vec<GpsPoint>,

//     segment_length: f64,
// ) -> Vec<Vec<GpsPoint>> {
//     let mut segments = Vec::new();
 
//     let mut current_segment = Vec::new();
//     let mut total_distance =0.0;

//     for i in 0..gps_points.len() - 1 {
//         let start = &gps_points[i];
//         let end = &gps_points[i + 1];
//         let distance = haversine_distance(start.lat, start.lon, end.lat, end.lon);

//         if total_distance + distance > segment_length {
//             segments.push(current_segment.clone());
//             // current_segment.clear();
//             // total_distance = 0.0;
//         }

//         current_segment.push(start.clone());
//         total_distance +=distance;
//     }

//     if !current_segment.is_empty() {
//         segments.push(current_segment);
//         println!("{:?}", segments);
//     }

//     segments
// }

// fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
//     const EARTH_RADIUS: f64 = 6371.0;
//     let delta_lat = (lat2 - lat1).to_radians();
//     let delta_lon = (lon2 - lon1).to_radians();

//     let a = (delta_lat / 2.0).sin().powi(2)
//         + lat1.to_radians().cos() * lat2.to_radians().cos() * (delta_lon / 2.0).sin().powi(2);
//     let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

//     EARTH_RADIUS * c
// }
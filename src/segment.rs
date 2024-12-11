use gpx::Waypoint;
use quick_xml::name::QName;
use quick_xml::{events::Event, Reader};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct GpsPoint {
    pub lat: f64,
    pub lon: f64,
    pub _time: u64,
}

pub fn parse_gpx(gpx_data: &str) -> Result<Vec<GpsPoint>, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_str(gpx_data);
    reader.config_mut().trim_text(true);
    let mut gps_points: Vec<GpsPoint> = Vec::new();

    let mut lat = None;
    let mut lon = None;
    let mut time = None;

    while let Ok(event) = reader.read_event() {
        match event {
            Event::Start(ref e) if e.name() == QName(b"trkpt") => {
                lat = e
                    .attributes()
                    .find(|attr| attr.as_ref().unwrap().key == QName(b"lat"))
                    .and_then(|attr| String::from_utf8_lossy(&attr.unwrap().value).parse().ok());
                lon = e
                    .attributes()
                    .find(|attr| attr.as_ref().unwrap().key == QName(b"lon"))
                    .and_then(|attr| String::from_utf8_lossy(&attr.unwrap().value).parse().ok());
            },
            Event::End(ref e) if e.name() == QName(b"trkpt") => {
                if let (Some(lat), Some(lon)) = (lat, lon) {
                    gps_points.push(GpsPoint { lat, lon, _time: time.unwrap_or(0) });
                }
            },
            Event::Eof => {
                break;
            },
            _ => {},
        }
    }

    Ok(gps_points)
}



mod air;
mod gps;
mod prover;
mod segment;
mod trace;
mod utils;
mod verifier;


use std::io::BufWriter;

use std::{fmt, io::BufReader};
use std::fs::File;

use actix_web::cookie::time;
use chrono::{DateTime, Duration, Utc};
use gps::PublicInputs;
use gpx::{write as write_gpx, Gpx, Time, Waypoint};
use prover::GpsProver;
use segment::parse_gpx;
use serde::Serialize;
use trace::{build_gps_trace_from_gpx,  build_gps_trace_from_points};
use verifier::verify_gps_trip;
use winterfell::{FieldExtension, ProofOptions, Prover, Trace};
use serde::ser::Serializer;
use winterfell::math::ToElements;
use time::OffsetDateTime;




const _START_TIME: &str = "10:00 AM";
const END_DURATION_HOURS: i64 = 6;
const INPUT_FILE: &str = "dushanbe.gpx";
const OUTPUT_FILE: &str = "output.gpx";

// Implement display for PublicInputs for better debugging
impl fmt::Display for PublicInputs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "lat: {}, lon: {}, next_lat: {}, next_lon: {}",
            self.lat, self.lon, self.next_lat, self.next_lon
        )
    }
}

// Convert PublicInputs to serializable elements (for debugging and serialization)
impl PublicInputs {
    fn to_serializable_elements(&self) -> Vec<String> {
        let elements = self.to_elements();
        elements.into_iter().map(|e| format!("{:?}", e)).collect()
    }
}

// Implement serialization for PublicInputs
impl Serialize for PublicInputs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let elements: Vec<String> = self.to_serializable_elements();
        serializer.serialize_some(&elements)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let file = File::open(INPUT_FILE).unwrap();
    let reader = BufReader::new(file);
    let mut gpx = gpx::read(reader)?;

    println!("Number of tracks: {}", gpx.tracks.len());
    if gpx.tracks.is_empty() {
        return Err("GPX файл не содержит треков.".into());
    }

    // Check tracks and their segments
    for (i, track) in gpx.tracks.iter().enumerate() {
        println!("Track {} contains {} segments.", i + 1, track.segments.len());
        if track.segments.is_empty() {
            return Err(format!("Трек {} не содержит сегментов.", i + 1).into());
        }

        for (j, segment) in track.segments.iter().enumerate() {
            println!("Segment {} contains {} dots.", j + 1, segment.points.len());
            if segment.points.is_empty() {
                return Err(format!("Сегмент {} в треке {} не содержит точек.", j + 1, i + 1).into());
            }
        }
    }

    // Extract all waypoints from the tracks and segments
    let mut points: Vec<&mut Waypoint> = gpx
        .tracks
        .iter_mut()
        .flat_map(|track| {
            track.segments.iter_mut().flat_map(|segment| segment.points.iter_mut())
        })
        .collect();

    if points.is_empty() {
        return Err("GPX файл не содержит точек трека.".into());
    }

    // Calculate start and end times for the track
    let start_time = DateTime::parse_from_str("2024 Dec 12 10:00:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")?
        .with_timezone(&Utc);
    let end_time = start_time + Duration::hours(END_DURATION_HOURS);

    // Distribute time across points
    let time_interval = (end_time - start_time).num_seconds() as usize / points.len();
    
    // Assign timestamps to waypoints
    for (i, point) in points.iter_mut().enumerate() {
        let timestamp = start_time + Duration::seconds((i * time_interval) as i64);
        point.time = Some(Time::from(OffsetDateTime::from_unix_timestamp(timestamp.timestamp())?));
    }

    // Write updated GPX data to output file
    let output = File::create(OUTPUT_FILE).map_err(|e| format!("Не удалось создать файл '{}': {}", OUTPUT_FILE, e))?;
    let writer = BufWriter::new(output);
    
    write_gpx(&gpx, writer).map_err(|e| format!("Ошибка записи файла '{}': {}", OUTPUT_FILE, e))?;
    println!("The file was saved successfully.: {}", OUTPUT_FILE);

    let file = File::open(OUTPUT_FILE)?;
    let reader = std::io::BufReader::new(file);
    let gpx = gpx::read(reader)?;

    let options = ProofOptions::new(
        32, // number of queries
        16, // blowup factor
        0,  // grinding factor
        FieldExtension::None,
        8,   // FRI folding factor
        127, // FRI remainder max degree
    );

    let trace = match build_gps_trace_from_gpx(gpx.clone()) {
        Ok(trace) => trace,
        Err(e) => return Err(e),  // Handle error
    };
    // let proof = prover.get_pub_inputs(&trace);

    let prover = GpsProver::new(options);
    // let proof = prover.get_pub_inputs(&trace);
    // let proof_bytes = serde_json::to_vec(&proof)?;
    // println!("Proof size: {:.1} KB", proof_bytes.len() as f64);

    let gpx_data = std::fs::read_to_string(INPUT_FILE)?;
    let gps_points = parse_gpx(&gpx_data)?;
    println!("Total points: {}", gps_points.len());



  
let chunks = gps_points.chunks(1024);
println!("Total segments: {}", chunks.len());
// for (i, points) in chunks.enumerate() {
//     println!("Обрабатывается сегмент {}...", i + 1);

//     // Генерируем трейс для сегмента
//     let trace = build_gps_trace_from_points(points.to_vec())?;
//     let public_inputs = prover.get_pub_inputs(&trace);
//     // Генерация доказательства
//     match prover.prove(trace) {
//         Ok(proof) => {
            
//             println!("Доказательство для сегмента {} успешно сгенерировано.", i + 1);
//             verify_gps_trip(
//                 public_inputs.lat,
//                 public_inputs.lon,
//                 public_inputs.next_lat,
//                 public_inputs.next_lon,
//                 public_inputs.time,
//                 public_inputs.next_time,
//                 proof,
//             );
//         }
//         Err(e) => {
//             println!("Ошибка при генерации доказательства: {}", e);
//         }
//     }

 
// }


for (i, points) in chunks.enumerate() {
    println!("Segment is being processed {}...", i + 1);

    // Генерируем трейс для сегмента
    let trace = build_gps_trace_from_points(points.to_vec())?;
    let public_inputs = prover.get_pub_inputs(&trace);

    // Генерация доказательства
    match prover.prove(trace) {
        Ok(proof) => {
            println!("Proof for segment {} successfully generated.", i + 1);

            // Сериализуем доказательство и считаем его размер
            let proof_bytes = proof.to_bytes();


            let proof_size_kb = proof_bytes.len() as f64 / 1024.0;

            println!(
                "Proof size for segment {}: {:.2} KB",
                i + 1,
                proof_size_kb
            );

            // Верификация доказательства
            verify_gps_trip(
                public_inputs.lat,
                public_inputs.lon,
                public_inputs.next_lat,
                public_inputs.next_lon,
                public_inputs.time,
                public_inputs.next_time,
                proof,
            );
        }
        Err(e) => {
            println!("Ошибка при генерации доказательства: {}", e);
        }
    }
}




    Ok(())
}





// mod air;
// mod gps;
// mod prover;
// mod segment;
// mod trace;
// mod utils;
// mod verifier;


// use std::io::BufWriter;

// use std::{fmt, io::BufReader};
// use std::fs::File;

// use actix_web::cookie::time;
// use chrono::{DateTime, Duration, Utc};
// use gps::PublicInputs;
// use gpx::{Waypoint, write as write_gpx, Time};
// use prover::GpsProver;
// use segment::parse_gpx;
// use serde::Serialize;
// use trace::{build_gps_trace_from_gpx,  write_trace_to_file};
// use verifier::verify_gps_trip;
// use winterfell::{FieldExtension, ProofOptions, Prover};
// use serde::ser::Serializer;
// use winterfell::math::ToElements;
// use time::OffsetDateTime;




// const _START_TIME: &str = "10:00 AM";
// const END_DURATION_HOURS: i64 = 6;
// const INPUT_FILE: &str = "dushanbe.gpx";
// const OUTPUT_FILE: &str = "output.gpx";

// // Implement display for PublicInputs for better debugging
// impl fmt::Display for PublicInputs {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "lat: {}, lon: {}, next_lat: {}, next_lon: {}",
//             self.lat, self.lon, self.next_lat, self.next_lon
//         )
//     }
// }

// // Convert PublicInputs to serializable elements (for debugging and serialization)
// impl PublicInputs {
//     fn to_serializable_elements(&self) -> Vec<String> {
//         let elements = self.to_elements();
//         elements.into_iter().map(|e| format!("{:?}", e)).collect()
//     }
// }

// // Implement serialization for PublicInputs
// impl Serialize for PublicInputs {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let elements: Vec<String> = self.to_serializable_elements();
//         serializer.serialize_some(&elements)
//     }
// }

// fn main() -> Result<(), Box<dyn std::error::Error>> {

//     let file = File::open(INPUT_FILE).unwrap();
//     let reader = BufReader::new(file);
//     let mut gpx = gpx::read(reader)?;

//     println!("Number of tracks: {}", gpx.tracks.len());
//     if gpx.tracks.is_empty() {
//         return Err("GPX файл не содержит треков.".into());
//     }

//     // Check tracks and their segments
//     for (i, track) in gpx.tracks.iter().enumerate() {
//         println!("Track {} contains {} segments.", i + 1, track.segments.len());
//         if track.segments.is_empty() {
//             return Err(format!("Трек {} не содержит сегментов.", i + 1).into());
//         }

//         for (j, segment) in track.segments.iter().enumerate() {
//             println!("Segment {} contains {} dots", j + 1, segment.points.len());
//             if segment.points.is_empty() {
//                 return Err(format!("Сегмент {} в треке {} не содержит точек.", j + 1, i + 1).into());
//             }
//         }
//     }

//     // Extract all waypoints from the tracks and segments
//     let mut points: Vec<&mut Waypoint> = gpx
//         .tracks
//         .iter_mut()
//         .flat_map(|track| {
//             track.segments.iter_mut().flat_map(|segment| segment.points.iter_mut())
//         })
//         .collect();

//     if points.is_empty() {
//         return Err("GPX файл не содержит точек трека.".into());
//     }

//     // Calculate start and end times for the track
//     let start_time = DateTime::parse_from_str("2024 Dec 12 10:00:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")?
//         .with_timezone(&Utc);
//     let end_time = start_time + Duration::hours(END_DURATION_HOURS);

//     // Distribute time across points
//     let time_interval = (end_time - start_time).num_seconds() as usize / points.len();
    
//     // Assign timestamps to waypoints
//     for (i, point) in points.iter_mut().enumerate() {
//         let timestamp = start_time + Duration::seconds((i * time_interval) as i64);
//         point.time = Some(Time::from(OffsetDateTime::from_unix_timestamp(timestamp.timestamp())?));
//     }

//     // Write updated GPX data to output file
//     let output = File::create(OUTPUT_FILE).map_err(|e| format!("Не удалось создать файл '{}': {}", OUTPUT_FILE, e))?;
//     let writer = BufWriter::new(output);
    
//     write_gpx(&gpx, writer).map_err(|e| format!("Ошибка записи файла '{}': {}", OUTPUT_FILE, e))?;
//     println!("The file was saved successfully.: {}", OUTPUT_FILE);

//     let file = File::open(OUTPUT_FILE)?;
//     let reader = std::io::BufReader::new(file);
//     let gpx = gpx::read(reader)?;

//     let options = ProofOptions::new(
//         32, // number of queries
//         16, // blowup factor
//         0,  // grinding factor
//         FieldExtension::None,
//         8,   // FRI folding factor
//         127, // FRI remainder max degree
//     );

//     let trace = build_gps_trace_from_gpx(&gpx);
//     if let Err(e) = write_trace_to_file(&trace, "trace_output.csv") {
//         eprintln!("Error writing to file: {}", e);
//     }
//     let gpx_data = std::fs::read_to_string(INPUT_FILE)?;
//     let gps_points = parse_gpx(&gpx_data)?;
//     println!("Total points: {}", gps_points.len());

//     let segments = gps_points.chunks(1024).collect::<Vec<_>>();
//     println!("Total segments: {}", segments.len());

//     let prover = GpsProver::new(options);
//     let proof = prover.get_pub_inputs(&trace);
//     let proof_bytes = serde_json::to_vec(&proof)?;
//     println!("Proof size: {:.1} KB", proof_bytes.len() as f64);
//     // println!("Proof size: {:.1} KiB", proof_bytes.len() as f64 / 1024.0);

//     // Шаг 5: Генерация публичных данных на основе трассировки
//     let public_inputs = prover.get_pub_inputs(&trace);

//     // Шаг 6: Генерация доказательства
//     match prover.prove(trace) {
//         Ok(proof) => {
//             println!("Proof generated successfully.");
//             // Шаг 7: Верификация доказательства
//             verify_gps_trip(
//                 public_inputs.lat,
//                 public_inputs.lon,
//                 public_inputs.next_lat,
//                 public_inputs.next_lon,
//                 public_inputs.time,
//                 public_inputs.next_time,
//                 proof,
//             );
//         },
//         Err(e) => {
//             println!("Ошибка при генерации доказательства: {}", e);
//         },
//     }
//     Ok(())
// }


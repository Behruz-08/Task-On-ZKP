
mod trace;
mod gps;
mod air;
mod prover;
mod utils;
mod verifier;

use std::fs::File;
use trace::build_gps_trace_from_gpx;
use utils::calculate_distance;
use verifier::verify_gps_trip;
use winterfell::{math::StarkField, FieldExtension, ProofOptions, Prover};


use prover::GpsProver;

fn main() -> Result<(), Box<dyn std::error::Error>> {

let n = 8;
   
    let file = File::open("./gps_data.gpx")?;
    let reader = std::io::BufReader::new(file);

    let gpx = gpx::read(reader)?;

    let trace = build_gps_trace_from_gpx( &gpx);

    let options = ProofOptions::new(
        32, // number of queries
        16,  // blowup factor
        0,  // grinding factor
        FieldExtension::None,
        8,   // FRI folding factor
        127, // FRI remainder max degree
    );
let prover = GpsProver::new(options);

let proof = prover.get_pub_inputs(&trace);
println!("Proof generated: {:?}", proof);

    println!("Трассировка:");
    for i in 0..n {
        let lat = trace.get(0, i).as_int() as f64 / 10_000_000.0;
      
        let lon = trace.get(1, i).as_int() as f64 / 10_000_000.0;
        
        let time = trace.get(2, i).as_int();
     
        if i < n - 1 {
           
            let next_lat = trace.get(0, i + 1).as_int() as f64 / 10_000_000.0;
            let next_lon = trace.get(1, i + 1).as_int() as f64 / 10_000_000.0;
            let distance = calculate_distance(lat, lon, next_lat, next_lon);
           
            println!(
                "Шаг {}: Широта: {:.7}, Долгота: {:.7}, Время: {} сек, Расстояние до след.: {:.2} м",
                i, lat, lon, time, distance
            );
        } else {
            println!(
                "Шаг {}: Широта: {:.7}, Долгота: {:.7}, Время: {} сек",
                i, lat, lon, time
            );
        }
    }

       
        // Шаг 5: Генерация публичных данных на основе трассировки
        let public_inputs = prover.get_pub_inputs(&trace);
       
        // Шаг 6: Генерация доказательства
        match prover.prove(trace) {
            Ok(proof) => {
                println!("Доказательство успешно сгенерировано.");
                // Шаг 7: Верификация доказательства
                verify_gps_trip(
                    public_inputs.lat,
                    public_inputs.lon,
                    public_inputs.next_lat,
                    public_inputs.next_lon,
                    proof,
                );
          
            }
            Err(e) => {
                println!("Ошибка при генерации доказательства: {}", e);
            }
        }

    Ok(())


}

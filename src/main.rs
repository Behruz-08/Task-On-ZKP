
mod trace;
mod gps;
mod air;
mod prover;
mod utils;
mod verifier;


use std::fs::File;
use gps::PublicInputs;
use prover::GpsProver;
use serde::Serialize;
use trace::{build_gps_trace_from_gpx, display_trace};
use verifier::verify_gps_trip;
use winterfell::{ FieldExtension, ProofOptions, Prover};
use std::fmt;
use winterfell::math::ToElements;
use serde::ser::Serializer;


impl fmt::Display for PublicInputs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lat: {}, lon: {}, next_lat: {}, next_lon: {}", self.lat, self.lon, self.next_lat, self.next_lon)
    }
}


impl PublicInputs {
    fn to_serializable_elements(&self) -> Vec<String> {
        let elements = self.to_elements();
        elements.into_iter().map(|e| format!("{:?}", e)).collect()
    }
}



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

    let file = File::open("./gps_data.gpx")?;
    let reader = std::io::BufReader::new(file);
    let gpx = gpx::read(reader)?;
    
    
    let options = ProofOptions::new(
        32, // number of queries
        16,  // blowup factor
        0,  // grinding factor
        FieldExtension::None,
        8,   // FRI folding factor
        31, // FRI remainder max degree
    );

    let trace = build_gps_trace_from_gpx(&gpx);

     let prover = GpsProver::new(options);
     let proof= prover.get_pub_inputs(&trace);
     let proof_bytes = serde_json::to_vec(&proof)?;
     println!("Proof size: {:.1} KB", proof_bytes.len() as f64 );

     display_trace(&trace);
       
        // Шаг 5: Генерация публичных данных на основе трассировки
        let public_inputs = prover.get_pub_inputs(&trace);
 
        // Шаг 6: Генерация доказательства
        match prover.prove(trace) {
            Ok(proof) => {
           
                // println!("Доказательство успешно сгенерировано.");
                // Шаг 7: Верификация доказательства
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
       
    Ok(())


  


  
}

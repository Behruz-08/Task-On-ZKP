use std::time::Instant;

use winterfell::{verify, AcceptableOptions, Proof};
use crate::air::{GpsAir, PublicInputs};
use winter_crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree};
use winterfell::math::fields::f128::BaseElement;

type Blake3 = Blake3_256<BaseElement>;
#[allow(dead_code)]
type VC = MerkleTree<Blake3>;

pub fn verify_gps_trip(
    lat: BaseElement,
    lon: BaseElement,
    next_lat: BaseElement,
    next_lon: BaseElement,
    time: BaseElement,
    next_time: BaseElement,
    proof: Proof,
    ) {
    let min_opts: AcceptableOptions = AcceptableOptions::MinConjecturedSecurity(95);
    let pub_inputs: PublicInputs = PublicInputs {
       lat,
        lon,
        next_lat,
        next_lon,
        time,
        next_time
    };
    let now: Instant = Instant::now();
    match verify::<GpsAir, Blake3, DefaultRandomCoin<Blake3>, VC>(proof, pub_inputs, &min_opts) {
        Ok(_) => println!( "Proof verified in {:.1} ms",now.elapsed().as_micros() as f64/1000f64 ),
        Err(e) => {
            println!("Ошибка верификации: {:?}", e);
            eprintln!("Подробности ошибки: {:?}", e);
        },
    }
}
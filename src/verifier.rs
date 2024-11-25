use winterfell::{verify, AcceptableOptions, Proof};
use crate::air::{GpsAir, PublicInputs};
use winter_crypto::{hashers::Blake3_256, DefaultRandomCoin};
use winterfell::math::fields::f128::BaseElement;

type Blake3 = Blake3_256<BaseElement>;


pub fn verify_gps_trip<VC: winter_crypto::VectorCommitment<winter_crypto::hashers::Blake3_256<winterfell::math::fields::f128::BaseElement>>>(
    start_lat: BaseElement,
    start_lon: BaseElement,
    end_lat: BaseElement,
    end_lon: BaseElement,
    proof: Proof,
) {
    let options = AcceptableOptions::MinConjecturedSecurity(95);
    let public_inputs = PublicInputs {
        start_lat,
        start_lon,
        end_lat,
        end_lon,
    };

    match verify::<GpsAir, Blake3, DefaultRandomCoin<Blake3>, VC>(proof, public_inputs, &options) {
        Ok(_) => println!("Доказательство успешно проверено."),
        Err(e) => println!("Ошибка проверки: {:?}", e),
    }
}

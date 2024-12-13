pub use crate::gps::PublicInputs;
use winterfell::math::{fields::f128::BaseElement, FieldElement};
use winterfell::{Air, AirContext, Assertion, EvaluationFrame, ProofOptions, TraceInfo};

use winterfell::TransitionConstraintDegree;

pub struct GpsAir {
    context: AirContext<BaseElement>,
    lat: BaseElement,
    lon: BaseElement,
    next_lat: BaseElement,
    next_lon: BaseElement,
}

impl Air for GpsAir {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;
    type GkrProof = ();
    type GkrVerifier = ();

    fn new(trace_info: TraceInfo, pub_inputs: PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(4, trace_info.width());

        let degrees = vec![
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
        ];

        let num_assertions = 4;

        GpsAir {
            context: AirContext::new(trace_info, degrees, num_assertions, options),
            lat: pub_inputs.lat,
            lon: pub_inputs.lon,
            next_lat: pub_inputs.next_lat,
            next_lon: pub_inputs.next_lon,
        }
    }

    fn evaluate_transition<E: FieldElement + From<Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        _periodic_values: &[E],
        result: &mut [E],
    ) {
        let current = frame.current();
        let next = frame.next();
        result[0] = next[2] - (current[2] + next[3]);
        result[1] = FieldElement::ZERO;
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let last_step = self.trace_length() - 1;
        vec![
            Assertion::single(0, 0, self.lat),
            Assertion::single(1, 0, self.lon),
            Assertion::single(0, last_step, self.next_lat),
            Assertion::single(1, last_step, self.next_lon),
        ]
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }
}

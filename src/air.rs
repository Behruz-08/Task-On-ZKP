use winterfell::{
        math::{fields::f128::BaseElement, FieldElement,  ToElements},
    Air, AirContext, Assertion,  EvaluationFrame, ProofOptions,
    TraceInfo, TransitionConstraintDegree,
};



#[derive(Debug)]
pub struct PublicInputs {
    pub start_lat: BaseElement,
    pub start_lon: BaseElement,
    pub end_lat: BaseElement,
    pub end_lon: BaseElement,
}

impl ToElements<BaseElement> for PublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        vec![self.start_lat, self.start_lon, self.end_lat, self.end_lon]
    }
}

pub struct GpsAir {
    context: AirContext<BaseElement>,
    start_lat: BaseElement,
    start_lon: BaseElement,
    end_lat: BaseElement,
    end_lon: BaseElement,
}

impl Air for GpsAir {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;
    type GkrProof = ();
    type GkrVerifier = ();

    fn new(trace_info: TraceInfo, pub_inputs: PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(2, trace_info.width());

        let degrees = vec![
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
        ];
        let num_assertions = 4;

        GpsAir {
            context: AirContext::new(trace_info, degrees, num_assertions, options),
            start_lat: pub_inputs.start_lat,
            start_lon: pub_inputs.start_lon,
            end_lat: pub_inputs.end_lat,
            end_lon: pub_inputs.end_lon,
        }
    }

    fn evaluate_transition<E: FieldElement + From<Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        _: &[E],
        result: &mut [E],
    ) {
        let current = frame.current();
        let next = frame.next();

        let next_lat = current[0] + E::from(BaseElement::ONE);
        let next_lon = current[1] + E::from(BaseElement::ONE);

        result[0] = next[0] - next_lat;
        result[1] = next[1] - next_lon;
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let last_step = self.trace_length() - 1;
        vec![
            Assertion::single(0, 0, self.start_lat),
            Assertion::single(1, 0, self.start_lon),
            Assertion::single(0, last_step, self.end_lat),
            Assertion::single(1, last_step, self.end_lon),
        ]
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }
}

use crate::{air::GpsAir, gps::PublicInputs};
use winter_crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree};
use winterfell::{math::fields::f128::BaseElement, TraceTable};
use winterfell::{
    math::FieldElement, matrix::ColMatrix, CompositionPoly, CompositionPolyTrace,
    DefaultConstraintCommitment, DefaultConstraintEvaluator, DefaultTraceLde, PartitionOptions,
    ProofOptions, Prover, StarkDomain, Trace, TraceInfo, TracePolyTable,
};
type Blake3 = Blake3_256<BaseElement>;
type VC = MerkleTree<Blake3>;

#[derive(Debug)]
pub struct GpsProver {
    options: ProofOptions,
}

impl GpsProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

impl Prover for GpsProver {
    type BaseField = BaseElement;
    type Air = GpsAir;
    type Trace = TraceTable<BaseElement>;
    type HashFn = Blake3;
    type RandomCoin = DefaultRandomCoin<Blake3>;
    type TraceLde<E: FieldElement<BaseField = BaseElement>> = DefaultTraceLde<E, Blake3, VC>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = BaseElement>> =
        DefaultConstraintEvaluator<'a, GpsAir, E>;
    type VC = MerkleTree<Blake3>;
    type ConstraintCommitment<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintCommitment<E, Blake3, Self::VC>;
    fn get_pub_inputs(&self, trace: &Self::Trace) -> PublicInputs {
        let last_step = trace.length() - 1;

        PublicInputs {
            lat: trace.get(0, 0),
            lon: trace.get(1, 0),
            next_lat: trace.get(0, last_step),
            next_lon: trace.get(1, last_step),
            time: trace.get(2, 0),
            next_time: trace.get(2, last_step),
        }
    }

    fn new_trace_lde<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        trace_info: &TraceInfo,
        main_trace: &ColMatrix<Self::BaseField>,
        domain: &StarkDomain<Self::BaseField>,
        partition_options: PartitionOptions,
    ) -> (Self::TraceLde<E>, TracePolyTable<E>) {
        DefaultTraceLde::new(trace_info, main_trace, domain, partition_options)
    }

    fn new_evaluator<'a, E: FieldElement<BaseField = BaseElement>>(
        &self,
        air: &'a GpsAir,
        aux_rand_elements: Option<winterfell::AuxRandElements<E>>,
        composition_coefficients: winterfell::ConstraintCompositionCoefficients<E>,
    ) -> Self::ConstraintEvaluator<'a, E> {
        DefaultConstraintEvaluator::new(air, aux_rand_elements, composition_coefficients)
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }

    fn build_constraint_commitment<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        composition_poly_trace: CompositionPolyTrace<E>,
        num_constraint_composition_columns: usize,
        domain: &StarkDomain<Self::BaseField>,
        partition_options: PartitionOptions,
    ) -> (Self::ConstraintCommitment<E>, CompositionPoly<E>) {
        DefaultConstraintCommitment::new(
            composition_poly_trace,
            num_constraint_composition_columns,
            domain,
            partition_options,
        )
    }
}

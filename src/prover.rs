
use winter_crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree};
// use winterfell::FieldExtension;
use winterfell::{
    math::{fields::f128::BaseElement, FieldElement},
 DefaultConstraintEvaluator, DefaultTraceLde,
    ProofOptions, Prover,  Trace, 
    TraceTable
};
use crate::air::{GpsAir, PublicInputs};


type Blake3 = Blake3_256<BaseElement>;

pub struct GpsProver {
    options: ProofOptions,
}

impl GpsProver {
    pub fn new(options: ProofOptions) -> Self {
        Self { options }
    }
}

// impl Prover for GpsProver {
//     type BaseField = BaseElement;
//     type Air = GpsAir;
//     type Trace =TraceTable<BaseElement>;
//     type HashFn = Blake3;
//     type RandomCoin = DefaultRandomCoin<Blake3>;
//     type TraceLde<E: FieldElement<BaseField = BaseElement>> = DefaultTraceLde<E, Blake3>;
//     type ConstraintEvaluator<'a, E: FieldElement<BaseField = BaseElement>> =
//         DefaultConstraintEvaluator<'a, GpsAir, E>;
//         type VC = MerkleTree<Blake3>;

//     fn get_pub_inputs(&self, trace: &Self::Trace) -> PublicInputs {
//         let last_step = trace.length() - 1;
//         PublicInputs {
//             start_lat: trace.get(0, 0),
//             start_lon: trace.get(1, 0),
//             end_lat: trace.get(0, last_step),
//             end_lon: trace.get(1, last_step),
//         }
//     }

//     fn options(&self) -> &ProofOptions {
//         &self.options
//     }
// }

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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> PublicInputs {
        let last_step = trace.length() - 1;
       
        PublicInputs {
            start_lat: trace.get(0, 0),
            start_lon: trace.get(1, 0),
            end_lat: trace.get(0, last_step),
            end_lon: trace.get(1, last_step),
           
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
    
// use actix_web::web::trace;
use winter_crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree};
use winterfell::FieldExtension;
use winterfell::{
    math::{fields::f128::BaseElement, FieldElement, StarkField, ToElements},
    matrix::ColMatrix,
    Air, AirContext, Assertion, DefaultConstraintEvaluator, DefaultTraceLde, EvaluationFrame,
    PartitionOptions, ProofOptions, Prover, StarkDomain, Trace, TraceInfo, TracePolyTable,
    TraceTable, TransitionConstraintDegree,
};
use winterfell::{verify, AcceptableOptions, Proof};

type Blake3 = Blake3_256<BaseElement>;
type VC = MerkleTree<Blake3>;

// Генерация трассировки для GPS-сегментов (широта и долгота)
pub fn build_gps_trace(start_lat: f64, start_lon: f64, n: usize) -> TraceTable<BaseElement> {
    let trace_width: usize = 2; // Ширина трассировки: долгота и широта
    let mut trace: TraceTable<BaseElement> = TraceTable::new(trace_width, n);

    let scale_factor: f64 = 10_000_000.0; // используемый коэффициент масштабировани

    let lat_u128: u128 = (start_lat * scale_factor as f64) as u128;
    let lon_u128: u128 = (start_lon * scale_factor as f64) as u128;

    trace.fill(
        |state: &mut [BaseElement]| {
            // Заполняем начальные координаты
            state[0] = BaseElement::new(lat_u128); // Начальная широта
            state[1] = BaseElement::new(lon_u128); // Начальная долгота
        
        },
        |_, state: &mut [BaseElement]| {
            // Моделируем изменение координат (например, на 1 метр)
            let next_lat_u128: u128 = state[0].as_int() as u128 + 1;
            let next_lon_u128: u128 = state[1].as_int() as u128 + 1;
            
            state[0] = BaseElement::new(next_lat_u128); // Обновление широты
            state[1] = BaseElement::new(next_lon_u128); // Обновление долготы
           
           
        },
    );

    trace
}


// Публичные входные данные (начальные и конечные координаты)
#[derive(Debug)]
pub struct PublicInputs {
    start_lat: BaseElement,
    start_lon: BaseElement,
    end_lat: BaseElement,
    end_lon: BaseElement,
}

impl ToElements<BaseElement> for PublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        vec![self.start_lat, self.start_lon, self.end_lat, self.end_lon]
    }
}

// Air для GPS вычислений
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
        assert_eq!(2, trace_info.width()); // Два столбца: широта и долгота

        let degrees = vec![TransitionConstraintDegree::new(1),TransitionConstraintDegree::new(1)];
      
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
        _periodic_values: &[E],
        result: &mut [E],
    ) {
        
        let current= frame.current();
        let next = frame.next();

        let current_lat = current[0];
        let current_lon = current[1];

        let next_lat = current_lat + E::from(BaseElement::ONE);
        let next_lon = current_lon + E::from(BaseElement::ONE);
        
        result[0] = next[0] - next_lat; // Проверка широты
        result[1] = next[1] - next_lon; // Пров
       
       
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

// Проверка доказательства
pub fn verify_gps_trip(start_lat: BaseElement, start_lon: BaseElement, end_lat: BaseElement, end_lon: BaseElement, proof: Proof) {
    let min_opts: AcceptableOptions = AcceptableOptions::MinConjecturedSecurity(95);
    let pub_inputs: PublicInputs = PublicInputs {
        start_lat,
        start_lon,
        end_lat,
        end_lon,
    };

    match verify::<GpsAir, Blake3, DefaultRandomCoin<Blake3>, VC>(proof, pub_inputs, &min_opts) {
        Ok(_) => println!("yay! все хорошо!"),
        Err(e) => println!("Ошибка верификации: {:?}", e),
    }
}

// Прокси для генерации доказательства
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

fn main() {
    // Шаг 1: Определение начальных данных
    let start_lat = 55.7558; // Начальная широта
    let start_lon = 37.6173; // Начальная долгота
    let n = 8; // Количество шагов (для примера 8)

    // Шаг 2: Построение трассировки
    let trace = build_gps_trace(start_lat, start_lon, n);

    //Печать каждого шага трассировки
    for i in 0..n {
      

        println!(
            "Step {}: Широта: {:?}, Долгота: {:?}",
            i,
            trace.get(0, i),
            trace.get(1, i),


        );
    }

    //Шаг 3: Создание опций для доказательства

    let options = ProofOptions::new(
        32, // num_queries
        8,  // blowup_factor
        0,  // grinding_factor
        FieldExtension::None,
        8,   // fri_remainder_max_degree
        127, // максимальная степень для FRI остатка
    );

  

    // Шаг 4: Создание провера с опциями
    let prover = GpsProver::new(options);

    // Шаг 5: Генерация публичных данных на основе трассировки
    let public_inputs = prover.get_pub_inputs(&trace);
println!("{:?}", public_inputs);
    // Шаг 6: Генерация доказательства
    match prover.prove(trace) {
        Ok(proof) => {
            println!("Доказательство успешно сгенерировано.");

            // Шаг 7: Верификация доказательства
            verify_gps_trip(
                public_inputs.start_lat,
                public_inputs.start_lon,
                public_inputs.end_lat,
                public_inputs.end_lon,
                proof,
            );
        }
        Err(e) => {
            println!("Ошибка при генерации доказательства: {}", e);
        }
    }
}


use std::f64::consts::PI;
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


// Генерация трассировки для GPS-сегментов (широта, долгота, время)
pub fn build_gps_trace_with_time(
    start_lat: f64,
    start_lon: f64,
    start_time: u64,
    step_time: u64,
    n: usize,
) -> TraceTable<BaseElement> {
    let trace_width: usize = 3; // Ширина трассировки: широта, долгота, время
    let mut trace: TraceTable<BaseElement> = TraceTable::new(trace_width, n);

    let scale_factor: f64 = 10_000_000.0; // Коэффициент масштабирования для координат
    let lat_u128: u128 = (start_lat * scale_factor) as u128;
    let lon_u128: u128 = (start_lon * scale_factor) as u128;

    trace.fill(
        |state: &mut [BaseElement]| {
            state[0] = BaseElement::new(lat_u128); // Начальная широта
            state[1] = BaseElement::new(lon_u128); // Начальная долгота
            state[2] = BaseElement::new(start_time.into()); // Начальное время
        },
        |_step, state: &mut [BaseElement]| {
            // Широта и долгота изменяются на 1
            let next_lat_u128 = state[0].as_int() as u128 + 1;
            let next_lon_u128 = state[1].as_int() as u128 + 1;
            let next_time = state[2].as_int() + step_time as u128;

            state[0] = BaseElement::new(next_lat_u128); // Обновление широты
            state[1] = BaseElement::new(next_lon_u128); // Обновление долготы
            state[2] = BaseElement::new(next_time); // Обновление времени
        },
    );

    trace
}



// Вычисление расстояния между координатами
fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371_000.0; // Радиус Земли в метрах
    let to_rad = |deg: f64| deg * PI / 180.0;

    let dlat = to_rad(lat2 - lat1);
    let dlon = to_rad(lon2 - lon1);
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    r * c // Расстояние в метрах
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
        assert_eq!(3, trace_info.width()); // Два столбца: широта и долгота

        let degrees =
         vec![TransitionConstraintDegree::new(1),
         TransitionConstraintDegree::new(1),TransitionConstraintDegree::new(1)];
      
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
        Ok(_) => println!("все хорошо!"),
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
    let start_lon     = 37.6173; // Начальная долгота
    let n=8; // Количество шагов (для примера 8)
    let start_time = 0; // Время начала в секундах
    let step_time = 1; // Шаг времени между измерениями


    // Построение трассировки
    let trace = build_gps_trace_with_time(start_lat, start_lon, start_time, step_time, n);

    println!("Трассировка:");
    for i in 0..n {
        let lat = trace.get(0, i).as_int() as f64 / 10_000_000.0;
        let lon = trace.get(1, i).as_int() as f64 / 10_000_000.0;
        let time = trace.get(2, i).as_int();

        // Если это не последний шаг, вычисляем расстояние до следующей точки
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



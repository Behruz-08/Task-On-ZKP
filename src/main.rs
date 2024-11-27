
use std::f64::consts::PI;
use chrono::{DateTime, Utc};
use time::OffsetDateTime;

use winter_crypto::{hashers::Blake3_256, DefaultRandomCoin, MerkleTree};

use winterfell::{
    math::{fields::f128::BaseElement, FieldElement, ToElements},
    matrix::ColMatrix,
    Air, AirContext, Assertion, DefaultConstraintEvaluator, DefaultTraceLde, EvaluationFrame,
    PartitionOptions, ProofOptions, Prover, StarkDomain, Trace, TraceInfo, TracePolyTable,
    TraceTable, TransitionConstraintDegree,
};
use winterfell::math::StarkField;


use std::fs::File;


use winterfell::{verify, AcceptableOptions, Proof};
use winterfell::DefaultConstraintCommitment;
use winterfell::CompositionPoly;
use winterfell::CompositionPolyTrace;

type Blake3 = Blake3_256<BaseElement>;
type VC = MerkleTree<Blake3>;


fn extract_timestamp(time: Option<gpx::Time>) -> i64 {
    match time {
        Some(t) => {
            let offset_datetime: OffsetDateTime = t.into(); // Преобразуем в OffsetDateTime

            // Используем unwrap() для извлечения значения из Option
            let chrono_time: DateTime<Utc> = DateTime::from_timestamp(offset_datetime.unix_timestamp(), offset_datetime.nanosecond())
                .expect("Failed to convert OffsetDateTime to DateTime<Utc>");

            chrono_time.timestamp() // Получаем timestamp
        },
        None => 0, // Если времени нет, возвращаем 0
    }
}

// Функция для создания трассировки из данных GPX
pub fn build_gps_trace_from_gpx(gpx: &gpx::Gpx) -> TraceTable<BaseElement> {
    let trace_width: usize = 3; // Ширина трассировки: широта, долгота, время
    let n = gpx.waypoints.len(); // Количество точек в маршруте
    let mut trace: TraceTable<BaseElement> = TraceTable::new(trace_width, n);

    let scale_factor: f64 = 10_000_000.0; // Коэффициент масштабирования для координат

    // Наполнение трассировки данными из GPX
    trace.fill(
        |state: &mut [BaseElement]| {
            let first_point = &gpx.waypoints[0];
            state[0] = BaseElement::new((first_point.point().y() * scale_factor) as u128); // Широта
            state[1] = BaseElement::new((first_point.point().x() * scale_factor) as u128); // Долгота
            state[2] = BaseElement::new(extract_timestamp(first_point.time) as u128); // Время (timestamp)
        },
        |step, state: &mut [BaseElement]| {
            if step < gpx.waypoints.len() - 1 {
                let current_point = &gpx.waypoints[step];
                let next_point = &gpx.waypoints[step + 1];

                let next_lat_u128 = (next_point.point().y() * scale_factor) as u128;
                let next_lon_u128 = (next_point.point().x() * scale_factor) as u128;
                let next_time = extract_timestamp(next_point.time) as u128;

                state[0] = BaseElement::new(next_lat_u128); // Обновление широты
                state[1] = BaseElement::new(next_lon_u128); // Обновление долготы
                state[2] = BaseElement::new(next_time); // Обновление времени
            }
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
        assert_eq!(3, trace_info.width()); 

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
    type HashFn = Blake3; // Указан конкретный тип Blake3
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
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _start_lat = 55.7558;
    let _start_lon = 37.6173;
    let n = 8;
    let _start_time = 0;
    let _step_time =1;

    let file = File::open("./gps_data.gpx")?;
    let reader = std::io::BufReader::new(file);

    let gpx = gpx::read(reader)?;
    

    // // Итерация по ссылкам, чтобы избежать перемещения
    // for waypoint in &gpx.waypoints {
    //     println!(
    //         "Широта: {}, Долгота: {}, Высота: {:?}, Время: {:?}",
    //         waypoint.point().y(), 
    //         waypoint.point().x(), 
    //         waypoint.elevation,
    //         waypoint.time
    //     );
    // }

    

   
    let trace = build_gps_trace_from_gpx( &gpx);

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

    Ok(())
}

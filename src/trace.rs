use winterfell::math::{FieldElement, StarkField};
use winterfell::{Trace, TraceTable};
use winterfell::math::fields::f128::BaseElement;
use crate::utils::{calculate_distance, extract_timestamp};



pub fn build_gps_trace_from_gpx(gpx: &gpx::Gpx) -> TraceTable<BaseElement> {
    let mut current_len = gpx.waypoints.len();

    
    if !current_len.is_power_of_two() {
        let next = current_len.next_power_of_two();
        println!("Current length is: {}. Next power of 2 is: {}", current_len, next);
        current_len = next;
    }
    let trace_width: usize = 4; // Ширина трассировки: широта, долгота, время
    let mut trace: TraceTable<BaseElement> = TraceTable::new(trace_width, current_len);

    let scale_factor: f64 = 10_000_000.0; // Коэффициент масштабирования для координат

    trace.fill(
        |state: &mut [BaseElement]| {
            let first_point = &gpx.waypoints[0];
            state[0] = BaseElement::new((first_point.point().y() * scale_factor) as u128);
            state[1] = BaseElement::new((first_point.point().x() * scale_factor) as u128);
            state[2] = BaseElement::new(extract_timestamp(first_point.time) as u128);
            state[3] = BaseElement::ZERO;
        },
        |step, state: &mut [BaseElement]| {
            if step < gpx.waypoints.len() - 1 {
                let next_point = &gpx.waypoints[step + 1];
                let next_lat_u128 = (next_point.point().y() * scale_factor) as u128;
                let next_lon_u128 = (next_point.point().x() * scale_factor) as u128;
                let next_time = extract_timestamp(next_point.time) as u128;

                let previous_timestamp = state[2].as_int() as u128;
                let time_diff = next_time - previous_timestamp;
                state[0] = BaseElement::new(next_lat_u128);
                state[1] = BaseElement::new(next_lon_u128);
                state[2] = BaseElement::new(next_time);
                state[3] = BaseElement::new(time_diff);
            }
        },
    );

    trace
}

pub fn display_trace(trace: &TraceTable<BaseElement>) {
    println!("Трассировка:");
    let n = trace.length();

    for i in 0..n {
        let lat = trace.get(0, i).as_int() as f64 / 10_000_000.0;
        let lon = trace.get(1, i).as_int() as f64 / 10_000_000.0;
        let time = trace.get(2, i).as_int(); 

        if i < n - 1 {
            let next_time = trace.get(2, i + 1).as_int();
            let next_lat = trace.get(0, i + 1).as_int() as f64 / 10_000_000.0;
            let next_lon = trace.get(1, i + 1).as_int() as f64 / 10_000_000.0;

            let time_diff = (next_time - time) as u128; // Разница времени
            let distance = calculate_distance(lat, lon, next_lat, next_lon);

            println!(
                "Шаг {}: Широта: {:.7}, Долгота: {:.7}, Расстояние до след.: {:.2} м, Время: {} сек, Разница времени: {} сек, ",
                i, lat, lon, distance, time, time_diff, 
            );
        } else {
            println!(
                "Шаг {}: Широта: {:.7}, Долгота: {:.7}, Время: {} сек, ",
                i, lat, lon, time, 
            );
        }
    }
}

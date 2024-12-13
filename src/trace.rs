

use crate::utils::{calculate_distance, extract_timestamp};
use gpx::Gpx;
use winterfell::math::fields::f128::BaseElement;
use winterfell::math::{FieldElement, StarkField};
use winterfell::{Trace, TraceTable};
use std::fs::File;
use std::io::{self, Write}; 

pub fn build_gps_trace_from_gpx(gpx: &Gpx) -> TraceTable<BaseElement> {
    let mut track_points = gpx
        .tracks
        .iter()
        .flat_map(|track| &track.segments)
        .flat_map(|segment| &segment.points)
        .map(|point| (point.point().y(), point.point().x(), extract_timestamp(point.time)))
        .collect::<Vec<_>>();

    let original_len = track_points.len();
    let mut current_len = original_len;

    if !current_len.is_power_of_two() {
        let next = current_len.next_power_of_two();
        println!(
            "Current length is: {}. Next power of 2 is: {}. Adding empty steps.",
            current_len, next
        );
        current_len = next;

        // // // Добавляем пустые точки с нулевыми значениями
        track_points.extend((0..(current_len - original_len)).map(|_| (0.0, 0.0, 0)));
    }

    let trace_width: usize = 4;
    let mut trace: TraceTable<BaseElement> = TraceTable::new(trace_width, current_len);

    let scale_factor: f64 = 10_000_000.0;

    trace.fill(
        |state: &mut [BaseElement]| {
            let first_point = &track_points[0];
            state[0] = BaseElement::new((first_point.0 * scale_factor) as u128);
            state[1] = BaseElement::new((first_point.1 * scale_factor) as u128 ) ;
           
            state[2] = BaseElement::new(first_point.2 as u128);
            state[3] = BaseElement::ZERO;
        },
        |step, state: &mut [BaseElement ] | {
            let next_point = &track_points[step + 1];
            let all_zeros: bool = next_point.0 == 0.0 && next_point.1 == 0.0;
            if all_zeros {
                state[0] = state[0];
                state[1] = state[1];
               
                state[2] = state[2];
                state[3] = BaseElement::ZERO;
            } else {
                
                let epsilon = 1e-6;
                let next_lat_u128 = if next_point.0.abs() > epsilon { (next_point.0 * scale_factor).abs() as u128 } else { 0 };
                let next_lon_u128 = if next_point.1.abs() > epsilon { (next_point.1 * scale_factor).abs() as u128 } else { 0 };

                let next_time = next_point.2 as u128;
                let previous_timestamp = state[2].as_int() as u128;
                let time_diff = next_time - previous_timestamp;

                state[0] = BaseElement::new(next_lat_u128);
                state[1] = BaseElement::new(next_lon_u128) ;
              
                state[2] = BaseElement::new(next_time);
                state[3] = BaseElement::new(time_diff);
            }
        },
    );

    trace
}

pub fn write_trace_to_file(trace: &TraceTable<BaseElement>, file_path: &str) -> io::Result<()> {
    let mut file = File::create(file_path)?; // Create or open the file for writing

    let n = trace.length();
    if n == 0 {
        writeln!(file, "Трассировка пуста.")?;
        return Ok(());
    }

    for i in 0..n {
        let lat = trace.get(0, i).as_int() as f64 / 10_000_000.0;
        let lon = trace.get(1, i).as_int() as f64 / 10_000_000.0;
        let time = trace.get(2, i).as_int();

        if i < n - 1 {
            let next_time = trace.get(2, i + 1).as_int();
            let next_lat = trace.get(0, i + 1).as_int() as f64 / 10_000_000.0;
            let next_lon = trace.get(1, i + 1).as_int() as f64 / 10_000_000.0;

            let time_diff = (next_time - time) as u128; // Time difference
            let distance = calculate_distance(lat, lon, next_lat, next_lon);

            writeln!(
                file,
                "Шаг {}: Широта: {:.7}, Долгота: {:.7}, Расстояние до след.: {:.2} м, Время: {} сек, Разница времени: {} сек",
                i + 1, lat, lon, distance, time, time_diff
            )?;
        } else {
            writeln!(
                file,
                "Шаг {}: Широта: {:.7}, Долгота: {:.7}, Время: {} сек",
                i + 1, lat, lon, time
            )?;
        }
    }

    Ok(())
}

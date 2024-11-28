use winterfell::TraceTable;
use winterfell::math::fields::f128::BaseElement;
use crate::utils::extract_timestamp;

// Функция для создания трассировки из данных GPX
pub fn build_gps_trace_from_gpx(gpx: &gpx::Gpx) -> TraceTable<BaseElement> {
    let trace_width: usize = 3; // Ширина трассировки: широта, долгота, время
    let n = gpx.waypoints.len(); // Количество точек в маршруте
    let mut trace: TraceTable<BaseElement> = TraceTable::new(trace_width, n);

    let scale_factor: f64 = 10_000_000.0; // Коэффициент масштабирования для координат

    trace.fill(
        |state: &mut [BaseElement]| {
            let first_point = &gpx.waypoints[0];
            state[0] = BaseElement::new((first_point.point().y() * scale_factor) as u128);
            state[1] = BaseElement::new((first_point.point().x() * scale_factor) as u128);
            state[2] = BaseElement::new(extract_timestamp(first_point.time) as u128);
         },
        |step, state: &mut [BaseElement]| {
            if step < gpx.waypoints.len() - 1 {
                let next_point = &gpx.waypoints[step + 1];
                let next_lat_u128 = (next_point.point().y() * scale_factor) as u128;
                let next_lon_u128 = (next_point.point().x() * scale_factor) as u128;
                let next_time = extract_timestamp(next_point.time) as u128;
                state[0] = BaseElement::new(next_lat_u128);
                state[1] = BaseElement::new(next_lon_u128);
                state[2] = BaseElement::new(next_time);
            }
        },
    );
    

    trace
}


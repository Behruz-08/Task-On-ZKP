use winterfell::math::StarkField;
use winterfell::math::fields::f128::BaseElement;
use winterfell::TraceTable;

// Генерация трассировки для GPS-сегментов
pub fn build_gps_trace(start_lat: f64, start_lon: f64, n: usize) -> TraceTable<BaseElement> {
    let trace_width: usize = 2; // Ширина трассировки: долгота и широта
    let mut trace: TraceTable<BaseElement> = TraceTable::new(trace_width, n);

    let scale_factor: f64 = 10_000_000.0;

    let lat_u128: u128 = (start_lat * scale_factor) as u128;
    let lon_u128: u128 = (start_lon * scale_factor) as u128;

    trace.fill(
        |state: &mut [BaseElement]| {
            state[0] = BaseElement::new(lat_u128);
            state[1] = BaseElement::new(lon_u128);
        },
        |_, state: &mut [BaseElement]| {
            let next_lat_u128: u128 = state[0].as_int() as u128 + 1;
            let next_lon_u128: u128 = state[1].as_int() as u128 + 1;

            state[0] = BaseElement::new(next_lat_u128);
            state[1] = BaseElement::new(next_lon_u128);
        },
    );

    trace
}

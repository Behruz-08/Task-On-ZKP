use winterfell::math::fields::f128::BaseElement;


#[derive(Debug)]
pub struct PublicInputs {
    pub lat: BaseElement,
    pub lon: BaseElement,
    pub next_lat: BaseElement,
    pub next_lon: BaseElement,
   
}

impl winterfell::math::ToElements<BaseElement> for PublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        vec![self.lat, self.lon, self.next_lat, self.next_lon]
    }
}

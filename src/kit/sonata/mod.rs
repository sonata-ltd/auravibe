pub mod button;
pub mod text;

#[derive(Clone)]
pub struct Sonata;

impl Sonata {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Sonata {
    fn default() -> Self {
        Self {}
    }
}

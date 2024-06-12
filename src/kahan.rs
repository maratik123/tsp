#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct KahanAdder {
    sum: f64,
    correction: f64,
}

impl KahanAdder {
    pub fn new(x: f64) -> Self {
        Self {
            sum: x,
            correction: 0.0,
        }
    }

    pub fn current_sum(&self) -> f64 {
        self.sum
    }

    pub fn result(self) -> f64 {
        self.sum
    }

    pub fn push_mut(&mut self, x: f64) {
        let y = x - self.correction;
        let sum = self.sum + y;
        self.correction = (sum - self.sum) - y;
        self.sum = sum;
    }

    pub fn push(self, x: f64) -> Self {
        let y = x - self.correction;
        let sum = self.sum + y;
        Self {
            correction: (sum - self.sum) - y,
            sum,
        }
    }

    pub fn push_and_result(self, x: f64) -> f64 {
        let y = x - self.correction;
        self.sum + y
    }
}

pub fn kahan_sum(it: impl Iterator<Item = f64>) -> f64 {
    it.fold(KahanAdder::default(), KahanAdder::push).result()
}

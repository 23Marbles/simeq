use num::BigRational;

struct AugmentedMatrix {
    height: usize,
    width: usize,

    values: Vec<Vec<BigRational>>,
    /// height
    augment: Vec<BigRational>,
}

impl AugmentedMatrix {
    pub fn swap_rows(&mut self, src: usize, dst: usize) {
        assert!(src < self.height);
        assert!(dst < self.height);
    }
}

pub struct EquationMatrix {
    height: usize,
    width: usize,

    augmented_matrix: AugmentedMatrix,

    /// width
    x: Vec<String>,
}

impl EquationMatrix {
    /// https://en.wikipedia.org/wiki/Gaussian_elimination
    pub fn solve() {}
}

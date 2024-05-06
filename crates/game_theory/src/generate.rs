use nalgebra::{DMatrix, DVector, Dyn, VecStorage, U1};
use rand::{
    distributions::uniform::{SampleRange, SampleUniform},
    Rng,
};

pub fn random_matrix<T: SampleUniform>(
    mut random: impl Rng,
    rows: usize,
    columns: usize,
    range: impl SampleRange<T> + Clone,
) -> DMatrix<T> {
    DMatrix::from_vec_storage(VecStorage::new(
        Dyn(rows),
        Dyn(columns),
        (0..rows * columns)
            .map(|_| random.gen_range(range.clone()))
            .collect(),
    ))
}

pub fn random_vector<T: SampleUniform>(
    mut random: impl Rng,
    length: usize,
    range: impl SampleRange<T> + Clone,
) -> DVector<T> {
    DVector::from_vec_storage(VecStorage::new(
        Dyn(length),
        U1,
        (0..length)
            .map(|_| random.gen_range(range.clone()))
            .collect(),
    ))
}

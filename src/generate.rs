use std::marker::PhantomData;
#[cfg(feature = "fastrand")]
use fastrand::u64;
use crate::Field;
#[cfg(feature = "rand")]
use rand::{Rng, RngCore};
#[cfg(feature = "rand")]
use rand::seq::{IteratorRandom, SliceRandom};
#[cfg(feature = "rand")]
use rand::prelude::{SeedableRng};
#[cfg(feature = "rand_chacha")]
use rand_chacha::{ChaCha8Rng};
use crate::field::Cell;

pub trait RandomMineSelector {
    fn get_mines_index(options: &FieldGenerationOptions) -> Vec<usize>;
}
pub struct FieldGenerator<M: RandomMineSelector>(PhantomData<M>);

#[cfg(all(not(feature = "rand_chacha"), not(feature = "rand"), feature = "fastrand"))]
pub type DefaultFieldGenerator = FieldGenerator<FastRandGenerator>;


#[cfg(all(not(feature="rand_chacha"), feature = "rand"))]
pub type DefaultFieldGenerator = FieldGenerator<ThreadRngFieldGenerator>;

#[cfg(feature = "rand_chacha")]
pub type DefaultFieldGenerator = FieldGenerator<ChaChaMineSelector>;

impl<M: RandomMineSelector> FieldGenerator<M> {
    pub fn generate(options: Option<FieldGenerationOptions>) -> Field {
        let options = options.unwrap_or_default();
        let mut elements = Vec::with_capacity(options.width * options.height);
        let mines = M::get_mines_index(&options);
        for i in 0..options.width * options.height {
            let mut cell = Cell::new();
            if mines.contains(&i) {
                cell.is_mine = true;
            }
            elements.push(cell);
        }
        Field::new(options.width, options.height, elements)
    }
}

#[cfg(feature = "fastrand")]
pub struct FastRandGenerator;

#[cfg(feature = "fastrand")]
impl RandomMineSelector for FastRandGenerator {
    fn get_mines_index(options: &FieldGenerationOptions) -> Vec<usize> {
        fastrand::seed(options.seed);
        let mut vec = (0..options.width * options.height).collect::<Vec<_>>();
        fastrand::shuffle(&mut vec);
        vec = vec.iter().take(options.mine_count).map(|u| *u).collect();
        vec
    }
}

#[cfg(feature = "rand")]
pub struct ThreadRngFieldGenerator {}

#[cfg(feature = "rand")]
impl RandomMineSelector for ThreadRngFieldGenerator {
    fn get_mines_index(options: &FieldGenerationOptions) -> Vec<usize> {
        let mut rng = rand::thread_rng();
        let mut vec: Vec<usize> = (0..options.width * options.height).into_iter().choose_multiple(&mut rng, options.mine_count);
        vec.shuffle(&mut rng);
        vec
    }
}

#[derive(Debug, Clone)]
pub struct FieldGenerationOptions {
    pub width: usize,
    pub height: usize,
    pub mine_count: usize,
    pub seed: u64,
}


impl Default for FieldGenerationOptions {
    #[cfg(feature = "fastrand")]
    fn default() -> Self {

        let seed = u64(..u64::MAX);
        Self {
            mine_count: 10,
            seed,
            width: 10,
            height: 10,
        }
    }

    #[cfg(all( not(feature = "fastrand"), any(feature = "rand")))]
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let seed = rng.next_u64();
        Self {
            mine_count: 10,
            seed,
            width: 10,
            height: 10,
        }
    }
}

#[cfg(feature = "rand_chacha")]
pub struct ChaChaMineSelector {}

#[cfg(feature = "rand_chacha")]
impl RandomMineSelector for ChaChaMineSelector {
    fn get_mines_index(options: &FieldGenerationOptions) -> Vec<usize> {
        let mut rng = ChaCha8Rng::seed_from_u64(options.seed);
        let mut vec: Vec<usize> = (0..options.width * options.height).into_iter().choose_multiple(&mut rng, options.mine_count);
        vec.shuffle(&mut rng);
        vec
    }
}

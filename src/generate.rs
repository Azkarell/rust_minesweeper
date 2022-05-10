use std::marker::PhantomData;
use crate::Field;
use rand::{Rng, RngCore};
use rand::seq::{IteratorRandom, SliceRandom};
use rand::prelude::{SeedableRng};
use rand_chacha::{ChaCha8Rng, ChaChaRng};
use crate::field::Cell;

pub trait RandomMineSelector {
    fn get_mines_index(options: &FieldGenerationOptions) -> Vec<usize>;
}

pub struct FieldGenerator<M: RandomMineSelector = ThreadRngFieldGenerator>(PhantomData<M>);

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


pub struct ThreadRngFieldGenerator {}

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

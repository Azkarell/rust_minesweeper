use crate::{CellHandle, Field, info, RevealResult};

pub trait Revealer {
    fn reveal_area(field: &mut Field, handle: CellHandle);
}

pub struct RevealerImpl;

impl Revealer for RevealerImpl {
    fn reveal_area(field: &mut Field, handle: CellHandle) {
        let neighbors = field.get_neighbors(handle);
        if neighbors.iter().filter(|&&n| field[n].is_mine).count() > 0 {
            return;
        }
        for neighbor in neighbors {
            match field.try_reveal(neighbor) {
                RevealResult::Mine => {
                    // do nothing
                }
                RevealResult::Empty(0) => {
                    field.reveal(neighbor);
                    Self::reveal_area(field, neighbor);
                }
                RevealResult::AlreadyRevealed => {
                    // do nothing
                }
                RevealResult::Empty(_) => {
                    field.reveal(neighbor);
                }
            }
        }
    }
}


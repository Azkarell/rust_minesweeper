use std::ops::{Index, IndexMut};
use crate::info;

#[derive(Debug, Clone, Copy)]
pub struct CellHandle {
    pub x: usize,
    pub y: usize,
}

impl ToString for CellHandle {
    fn to_string(&self) -> String {
        format!("{},{}", self.x, self.y)
    }
}

impl CellHandle {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl Default for CellHandle {
    fn default() -> Self {
        CellHandle { x: 0, y: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellState {
    Hidden,
    Marked(Mark),
    Revealed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mark {
    Empty,
    Mine,
}



#[derive(Debug, Clone)]
pub struct Cell {
    state: CellState,
    pub is_mine: bool,
    adjacent_mines: Option<usize>,
    pub is_dirty: bool,
}


impl Cell {
    pub fn new() -> Self {
        Cell {
            state: CellState::Hidden,
            is_mine: false,
            adjacent_mines: None,
            is_dirty: false,
        }
    }

    pub fn set_state(&mut self, state: CellState) {
        self.state = state;
        self.is_dirty = true;
    }

    pub fn get_state(&self) -> &CellState {
        &self.state
    }

}

impl Default for Cell {
    fn default() -> Self {
        Cell::new()
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub width: usize,
    pub height: usize,
    elements: Vec<Cell>,
    count_unrevealed: usize,
}

pub enum RevealResult {
    Mine,
    Empty(usize),
    AlreadyRevealed,
}

impl Field {
    pub fn new(width: usize, height: usize, cells: Vec<Cell>) -> Self {
        let count_unrevealed = cells.iter().filter(|c| c.state == CellState::Hidden).count();
        Self {
            width,
            height,
            elements: cells,
            count_unrevealed,
        }
    }

    pub fn get_adjacent_mines(&mut self, cell: CellHandle) -> usize {
        if let Some(adj) = self[cell].adjacent_mines {
            adj
        } else {
            let neighbors = self.get_neighbors(cell);
            let adj = neighbors.iter().filter(|&&c| self[c].is_mine).count();
            self[cell].adjacent_mines = Some(adj);
            adj
        }
    }
    
    pub fn is_won(&self) -> bool {
        self.elements.iter().filter(|c| !c.is_mine).all(|c| c.state == CellState::Revealed)
    }

    pub fn get_neighbors(&self, cell: CellHandle) -> Vec<CellHandle> {
        let mut ret = Vec::new();

        for x in  cell.x.saturating_sub(1)..=(cell.x + 1).clamp(0, self.width-1) {
            for y in cell.y.saturating_sub(1)..=(cell.y + 1).clamp(0, self.height -1) {
                if x == cell.x && y == cell.y {
                    continue;
                }
                ret.push(CellHandle::new(x, y));
            }
        }
        ret
    }

    pub fn get_handles(&self) -> Vec<CellHandle> {
        let mut ret = Vec::new();
        for x in 0..self.width {
            for y in 0..self.height {
                ret.push(CellHandle::new(x, y));
            }
        }
        ret
    }




    pub fn try_reveal(&self, cell: CellHandle) -> RevealResult {
        let c = &self[cell];
        if c.state == CellState::Revealed {
            return RevealResult::AlreadyRevealed;
        }
        if c.is_mine {
            return RevealResult::Mine;
        }

        if c.adjacent_mines.is_none() {
            let neighbors = self.get_neighbors(cell);
            let surrounding_mines = neighbors.iter().filter(|&&n| self[n].is_mine).count();
            return RevealResult::Empty(surrounding_mines);
        }

        RevealResult::Empty(c.adjacent_mines.expect("adjacent_mines should be set"))
    }

    pub fn reveal(&mut self, cell: CellHandle) -> RevealResult {
        let r = self.try_reveal(cell);
        match r {
            RevealResult::Mine => {
                self[cell].set_state(CellState::Revealed);
            }
            RevealResult::Empty(adjacent_mines) => {
                self[cell].set_state(CellState::Revealed);
                self[cell].adjacent_mines = Some(adjacent_mines);
                self.count_unrevealed -= 1;
            }
            RevealResult::AlreadyRevealed => {
                // do nothing
            }
        }
        r
    }

    pub fn toggle_mark(&mut self, cell: CellHandle) {
        let mut c = &mut self[cell];

        match c.state {
            CellState::Hidden => {
                c.set_state( CellState::Marked(Mark::Mine));
            }
            CellState::Marked(Mark::Mine) => {
                c.set_state( CellState::Marked(Mark::Empty));
            }
            CellState::Marked(Mark::Empty) => {
                c.set_state(CellState::Hidden);
            }
            _ => {
                // do nothing
            }
        }
    }
}


impl Index<usize> for Field {
    type Output = [Cell];
    fn index(&self, index: usize) -> &Self::Output {
        &self.elements[index * self.height..(index + 1) * self.height]
    }
}

impl IndexMut<usize> for Field {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.elements[index * self.height..(index + 1) * self.height]
    }
}


impl Index<CellHandle> for Field {
    type Output = Cell;
    fn index(&self, index: CellHandle) -> &Self::Output {
        &self[index.x][index.y]
    }
}

impl IndexMut<CellHandle> for Field {
    fn index_mut(&mut self, index: CellHandle) -> &mut Self::Output {
        &mut self[index.x][index.y]
    }
}
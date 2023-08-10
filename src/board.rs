use crate::board::solver::SolvingBoard;
use crate::log;
use std::collections::BTreeSet;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Unsettled = 255,
    Empty = 0,
    Bomb = 1,
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellState {
    Hidden = 0,
    Revealed = 1,
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq)]
pub struct Board {
    width: u32,
    height: u32,
    bombs: u32,
    cells: Vec<Cell>,
    stats: Vec<CellState>,
    numbers: Vec<u8>,
    pass_rate: f64,

    clue_cells: BTreeSet<usize>,
}

impl Board {
    pub fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }
    pub fn get_row_col(&self, index: usize) -> (u32, u32) {
        (index as u32 / self.width, index as u32 % self.width)
    }
    pub fn get_neighbours(&self, idx: usize) -> Vec<usize> {
        let mut neighbours = vec![];
        let (row, column) = self.get_row_col(idx);
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }
                if delta_row == self.height - 1 && row == 0 {
                    continue;
                }
                if delta_col == self.width - 1 && column == 0 {
                    continue;
                }
                if delta_row == 1 && row == self.height - 1 {
                    continue;
                }
                if delta_col == 1 && column == self.width - 1 {
                    continue;
                }
                let neighbour_row = (row + delta_row) % self.height;
                let neighbour_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbour_row, neighbour_col);
                neighbours.push(idx);
            }
        }
        neighbours
    }
}

#[wasm_bindgen]
impl Board {
    pub fn new(width: u32, height: u32, bombs: u32) -> Board {
        assert!(width > 0);
        assert!(height > 0);
        assert!(bombs > 0);
        assert!(bombs < width * height);
        let mut cells: Vec<_> = (0..width * height).map(|_| Cell::Unsettled).collect();
        let mut stats: Vec<_> = (0..width * height).map(|_| CellState::Hidden).collect();
        let mut numbers: Vec<_> = (0..width * height).map(|_| 255).collect();
        cells[width as usize + 1] = Cell::Empty;
        stats[width as usize + 1] = CellState::Revealed;
        numbers[width as usize + 1] = 1;
        Board {
            width,
            height,
            bombs,
            cells,
            stats,
            numbers,
            pass_rate: 1.0,
            clue_cells: BTreeSet::from([width as usize + 1]),
        }
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn bombs(&self) -> u32 {
        self.bombs
    }
    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }
    pub fn stats(&self) -> *const CellState {
        self.stats.as_ptr()
    }
    pub fn numbers(&self) -> *const u8 {
        self.numbers.as_ptr()
    }
    pub fn is_clue_cell(&self, idx: usize) -> bool {
        self.clue_cells.contains(&idx)
    }
    pub fn is_hint_cell(&self, idx: usize) -> bool {
        self.hint_cells().contains(&idx)
    }
    pub fn pass_rate(&self) -> f64 {
        self.pass_rate
    }
    pub fn rest_bombs(&self) -> u32 {
        self.bombs() - self.cells.iter().filter(|&&c| c == Cell::Bomb).count() as u32
    }
    pub fn rest_cells(&self) -> u32 {
        self.cells.iter().filter(|&&c| c == Cell::Unsettled).count() as u32
    }
    pub fn is_revealed(&self, row: u32, column: u32) -> bool {
        let idx = self.get_index(row, column);
        self.stats[idx] == CellState::Revealed
    }
    pub fn reveal(&self, row: u32, column: u32, mut number: u8) -> Option<Board> {
        let idx = self.get_index(row, column);
        if self.stats[idx] == CellState::Revealed {
            return None;
        }
        if self.cells[idx] == Cell::Bomb {
            let mut new_board = self.clone();
            new_board.stats[idx] = CellState::Revealed;
            return Some(new_board);
        }
        let mut new_board = self.clone();
        if self
            .get_neighbours(idx)
            .iter()
            .all(|&idx| self.cells[idx] != Cell::Unsettled)
        {
            number = 255;
        }
        let pass_rate = if self.cells[idx] == Cell::Unsettled {
            let rater = SolvingBoard::from(self.clone());
            rater.pass_rate(idx)
        } else {
            1.0
        };
        new_board.reveal_inner(row, column, number);
        let solver = SolvingBoard::from(new_board.clone());
        if solver.is_valid() {
            new_board.pass_rate *= pass_rate;
            new_board.solve();
            new_board.update_clues();
            Some(new_board)
        } else {
            None
        }
    }
    fn reveal_inner(&mut self, row: u32, column: u32, number: u8) {
        let idx = self.get_index(row, column);
        self.cells[idx] = Cell::Empty;
        self.stats[idx] = CellState::Revealed;
        self.numbers[idx] = number;
        if number != 255 {
            self.clue_cells.insert(idx);
        }
    }

    fn is_valid(&self) -> bool {
        let mut unsettled = 0;
        let mut bomb = 0;
        for (idx, &cell) in self.cells.iter().enumerate() {
            match cell {
                Cell::Empty => {
                    if self.stats[idx] == CellState::Revealed {
                        let num = self.numbers[idx];
                        if num == 255 {
                            continue;
                        }
                        let neighbours = self.get_neighbours(idx);
                        let mut n_bomb = 0;
                        let mut n_unsettled = 0;
                        for nei in neighbours {
                            match self.cells[nei] {
                                Cell::Bomb => {
                                    n_bomb += 1;
                                }
                                Cell::Unsettled => {
                                    n_unsettled += 1;
                                }
                                _ => {}
                            }
                        }
                        if n_bomb > num || n_bomb + n_unsettled < num {
                            return false;
                        }
                    }
                }
                Cell::Bomb => {
                    bomb += 1;
                }
                Cell::Unsettled => {
                    unsettled += 1;
                }
            }
        }
        bomb <= self.bombs && bomb + unsettled >= self.bombs
    }
    fn clue_cells(&self) -> Vec<usize> {
        self.clue_cells.iter().cloned().collect()
    }
    fn hint_cells(&self) -> Vec<usize> {
        let mut hint_cells = self
            .clue_cells()
            .iter()
            .flat_map(|&c| self.get_neighbours(c))
            .filter(|&c| self.cells[c] == Cell::Unsettled)
            .collect::<Vec<_>>();
        hint_cells.sort();
        hint_cells.dedup();
        hint_cells
    }
    pub fn solve(&mut self) {
        // trivial
        self.clue_cells().into_iter().for_each(|index| {
            let bombs = self
                .get_neighbours(index)
                .into_iter()
                .filter(|&nei| self.cells[nei] == Cell::Bomb)
                .count();
            let unsettles = self
                .get_neighbours(index)
                .into_iter()
                .filter(|&nei| self.cells[nei] == Cell::Unsettled)
                .count();
            if self.numbers[index] == bombs as u8 {
                self.get_neighbours(index).into_iter().for_each(|nei| {
                    if self.cells[nei] == Cell::Unsettled {
                        self.cells[nei] = Cell::Empty;
                    }
                });
            }
            if self.numbers[index] == (bombs + unsettles) as u8 {
                self.get_neighbours(index).into_iter().for_each(|nei| {
                    if self.cells[nei] == Cell::Unsettled {
                        self.cells[nei] = Cell::Bomb;
                    }
                });
            }
        });
        // brute
        self.hint_cells().into_iter().for_each(|index| {
            let mut tmp_board = self.clone();
            tmp_board.cells[index] = Cell::Bomb;
            tmp_board.stats[index] = CellState::Revealed;
            let solver = SolvingBoard::from(tmp_board.clone());
            if solver.valid_boards() == 0 {
                self.cells[index] = Cell::Empty;
            } else {
                let mut tmp_board = self.clone();
                tmp_board.cells[index] = Cell::Empty;
                tmp_board.stats[index] = CellState::Revealed;
                let solver = SolvingBoard::from(tmp_board.clone());
                if solver.valid_boards() == 0 {
                    self.cells[index] = Cell::Bomb;
                }
            }
        });
        // rest
        let bombs = self.cells.iter().filter(|&&c| c == Cell::Bomb).count() as u32;
        if bombs == self.bombs {
            self.cells.iter_mut().for_each(|c| {
                if *c == Cell::Unsettled {
                    *c = Cell::Empty;
                }
            });
        }
        let unsettles = self.cells.iter().filter(|&&c| c == Cell::Unsettled).count() as u32;
        if bombs + unsettles == self.bombs {
            self.cells.iter_mut().for_each(|c| {
                if *c == Cell::Unsettled {
                    *c = Cell::Bomb;
                }
            });
        }
    }
    fn update_clues(&mut self) {
        let mut clue_cells = self
            .clue_cells
            .iter()
            .filter(|&&c| {
                !self
                    .get_neighbours(c)
                    .iter()
                    .all(|&nei| self.cells[nei] != Cell::Unsettled)
            })
            .copied()
            .collect();
        log(&format!("clue_cells: {:?}", clue_cells));
        self.clue_cells = clue_cells;
        log(&format!("hint_cells: {:?}", self.hint_cells()));
    }
}

mod solver;

mod tests {
    use super::*;
    #[wasm_bindgen]
    pub fn test_board() {
        use Cell::*;
        use CellState::*;
        let board = Board {
            width: 3,
            height: 3,
            bombs: 1,
            cells: "....f...."
                .chars()
                .map(|c| match c {
                    '.' => Unsettled,
                    'f' => Bomb,
                    _ => unreachable!(),
                })
                .collect(),
            stats: "....rrrrr"
                .chars()
                .map(|c| match c {
                    '.' => Hidden,
                    'r' => Revealed,
                    _ => unreachable!(),
                })
                .collect(),
            numbers: ".....3221"
                .chars()
                .map(|c| match c {
                    '.' => 255,
                    '1' => 1,
                    '2' => 2,
                    '3' => 3,
                    _ => unreachable!(),
                })
                .collect(),
            pass_rate: 1.0,

            clue_cells: BTreeSet::new(),
        };
        assert!(board.is_valid());
    }
}

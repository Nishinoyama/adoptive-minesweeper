use super::*;
use crate::log;

pub struct SolvingBoard {
    board: Board,
}

impl From<Board> for SolvingBoard {
    fn from(board: Board) -> Self {
        SolvingBoard { board }
    }
}

fn combination(n: u32, r: u32) -> f64 {
    let mut p = 1.0;
    for i in 1..=n {
        p *= i as f64;
    }
    for i in 1..=r {
        p /= i as f64;
    }
    for i in 1..=(n - r) {
        p /= i as f64;
    }
    p
}

impl SolvingBoard {
    pub fn is_valid(&self) -> bool {
        self.valid_boards() > 0
    }
    pub fn pass_rate(&self, idx: usize) -> f64 {
        let hint_cells = self.board.hint_cells();
        // brute force search for n-bits
        let n = hint_cells.len();
        log(&format!("hint_cells: {:?}", hint_cells));
        let mut universe = 0.0;
        let mut valid = 0.0;
        (0..(1 << n)).for_each(|i| {
            let (bomb_cells, empty_cells) = {
                hint_cells.iter().enumerate().fold(
                    (vec![], vec![]),
                    |(mut bomb_cells, mut empty_cells), (j, &c)| {
                        if i & (1u64 << j) != 0 {
                            bomb_cells.push(c);
                        } else {
                            empty_cells.push(c);
                        }
                        (bomb_cells, empty_cells)
                    },
                )
            };
            let mut tmp_board = self.board.clone();
            bomb_cells.iter().for_each(|&c| {
                tmp_board.cells[c] = Cell::Bomb;
                tmp_board.stats[c] = CellState::Revealed;
            });
            empty_cells.iter().for_each(|&c| {
                tmp_board.cells[c] = Cell::Empty;
                tmp_board.stats[c] = CellState::Revealed;
            });
            if tmp_board.is_valid() {
                let sum = combination(tmp_board.rest_cells(), tmp_board.rest_bombs());
                universe += sum;
                if tmp_board.cells[idx] == Cell::Empty {
                    valid += sum;
                } else if tmp_board.cells[idx] == Cell::Unsettled {
                    valid += sum / tmp_board.rest_cells() as f64
                        * (tmp_board.rest_cells() - tmp_board.rest_bombs()) as f64;
                }
            }
        });
        log(&format!("valid: {}", valid));
        log(&format!("universe: {}", universe));
        log(&format!("pass_rate: {}", valid / universe));
        let rate = valid / universe;
        if rate.is_nan() {
            1.0
        } else {
            rate
        }
    }
    pub fn valid_boards(&self) -> usize {
        let hint_cells = self.board.hint_cells();
        // brute force search for n-bits
        let n = hint_cells.len();
        (0..(1 << n))
            .filter(|&i| {
                let (bomb_cells, empty_cells) = {
                    hint_cells.iter().enumerate().fold(
                        (vec![], vec![]),
                        |(mut bomb_cells, mut empty_cells), (j, &c)| {
                            if i & (1u64 << j) != 0 {
                                bomb_cells.push(c);
                            } else {
                                empty_cells.push(c);
                            }
                            (bomb_cells, empty_cells)
                        },
                    )
                };
                let mut tmp_board = self.board.clone();
                bomb_cells.iter().for_each(|&c| {
                    tmp_board.cells[c] = Cell::Bomb;
                    tmp_board.stats[c] = CellState::Revealed;
                });
                empty_cells.iter().for_each(|&c| {
                    tmp_board.cells[c] = Cell::Empty;
                    tmp_board.stats[c] = CellState::Revealed;
                });
                tmp_board.is_valid()
            })
            .count()
    }
}

use super::*;

pub struct SolvingBoard {
    board: Board,
}

impl From<Board> for SolvingBoard {
    fn from(board: Board) -> Self {
        SolvingBoard { board }
    }
}

fn combination(n: u32, r: u32) -> f64 {
    let mut dp = vec![vec![0.0; n as usize + 1]; n as usize + 1];
    dp[0][0] = 1.0;
    for i in 1..=n as usize {
        dp[i][0] = 1.0;
        for j in 1..=i {
            dp[i][j] = dp[i - 1][j - 1] + dp[i - 1][j];
        }
    }
    dp[n as usize][r as usize]
}

impl SolvingBoard {
    pub fn is_valid_on_watching(&self) -> bool {
        !self.valid_hint_watching_cell_patterns().is_empty()
    }
    pub fn is_valid(&self) -> bool {
        self.valid_boards() > 0
    }
    pub fn pass_rate(&self, idx: usize) -> f64 {
        return 1.0;
        let mut universe = 0.0;
        let mut valid = 0.0;
        self.valid_hint_cell_patterns()
            .iter()
            .filter_map(|(bomb_cells, empty_cells)| {
                let board = self.pattern_applied_board(empty_cells, bomb_cells);
                board.is_valid().then_some(board)
            })
            .for_each(|board| {
                let sum = combination(board.rest_cells(), board.rest_bombs());
                universe += sum;
                if board.cells[idx] == Cell::Empty {
                    valid += sum;
                } else if board.cells[idx] == Cell::Unsettled {
                    valid += sum / board.rest_cells() as f64
                        * (board.rest_cells() - board.rest_bombs()) as f64;
                }
            });
        let rate = valid / universe;
        if rate.is_nan() {
            1.0
        } else {
            rate
        }
    }
    pub fn valid_boards(&self) -> usize {
        self.valid_hint_cell_patterns().len()
    }

    fn pattern_applied_board(&self, empty_cells: &[usize], bomb_cells: &[usize]) -> Board {
        let mut tmp_board = self.board.clone();
        bomb_cells.iter().for_each(|&c| {
            tmp_board.cells[c] = Cell::Bomb;
            tmp_board.stats[c] = CellState::Revealed;
        });
        empty_cells.iter().for_each(|&c| {
            tmp_board.cells[c] = Cell::Empty;
            tmp_board.stats[c] = CellState::Revealed;
        });
        tmp_board
    }

    fn is_valid_with_cells(&self, empty_cells: &[usize], bomb_cells: &[usize]) -> bool {
        self.pattern_applied_board(empty_cells, bomb_cells)
            .is_valid()
    }

    pub fn valid_hint_watching_cell_patterns(&self) -> Vec<(Vec<usize>, Vec<usize>)> {
        let res =
            self.board
                .watching_hint_cells()
                .iter()
                .fold(vec![(vec![], vec![])], |cells, &i| {
                    cells
                        .into_iter()
                        .flat_map(|(bomb_cells, empty_cells)| {
                            let bomb_cells_added = vec![bomb_cells.clone(), vec![i]].concat();
                            let empty_cells_added = vec![empty_cells.clone(), vec![i]].concat();
                            let mut res = vec![];
                            if self.is_valid_with_cells(&empty_cells, &bomb_cells_added) {
                                res.push((bomb_cells_added, empty_cells));
                            }
                            if self.is_valid_with_cells(&empty_cells_added, &bomb_cells) {
                                res.push((bomb_cells, empty_cells_added));
                            }
                            res
                        })
                        .collect()
                });
        log(&format!("{:?}", res.len()));
        res
    }

    pub fn valid_hint_cell_patterns(&self) -> Vec<(Vec<usize>, Vec<usize>)> {
        let res = self
            .board
            .hint_cells()
            .iter()
            .fold(vec![(vec![], vec![])], |cells, &i| {
                cells
                    .into_iter()
                    .flat_map(|(bomb_cells, empty_cells)| {
                        let bomb_cells_added = vec![bomb_cells.clone(), vec![i]].concat();
                        let empty_cells_added = vec![empty_cells.clone(), vec![i]].concat();
                        let mut res = vec![];
                        if self.is_valid_with_cells(&empty_cells, &bomb_cells_added) {
                            res.push((bomb_cells_added, empty_cells));
                        }
                        if self.is_valid_with_cells(&empty_cells_added, &bomb_cells) {
                            res.push((bomb_cells, empty_cells_added));
                        }
                        res
                    })
                    .collect()
            });
        log(&format!("{:?}", res.len()));
        res
    }
}

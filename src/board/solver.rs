use super::*;

pub struct SolvingBoard {
    board: Board,
}

impl From<Board> for SolvingBoard {
    fn from(board: Board) -> Self {
        SolvingBoard { board }
    }
}

impl SolvingBoard {
    pub fn is_valid(&self) -> bool {
        self.valid_boards() > 0
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

mod utils;

use std::fmt::Display;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, qua_ms!");
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty = 0,
    Bomb = 1,
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellState {
    Hidden = 0,
    Revealed = 1,
    Flagged = 2,
    Questioned = 3,
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    stats: Vec<CellState>,
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn new(width: u32, height: u32, bombs: u32) -> Universe {
        assert!(width > 0);
        assert!(height > 0);
        assert!(bombs > 0);
        assert!(bombs < width * height);
        let mut cells = (0..width * height).map(|_| Cell::Empty).collect::<Vec<_>>();
        // put boms in random places
        for _ in 0..bombs {
            let mut idx = (js_sys::Math::random() * width as f64 * height as f64) as usize;
            while cells[idx] == Cell::Bomb {
                idx = (js_sys::Math::random() * width as f64 * height as f64) as usize;
            }
            cells[idx] = Cell::Bomb;
        }
        let stats = (0..width * height).map(|_| CellState::Hidden).collect();
        Universe {
            width,
            height,
            cells,
            stats,
        }
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }
    pub fn stats(&self) -> *const CellState {
        self.stats.as_ptr()
    }
    pub fn neighbour_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
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
                count += self.cells[idx] as u8;
            }
        }
        count
    }
    pub fn reveal(&self, row: u32, column: u32) -> Self {
        let mut new_universe = self.clone();
        new_universe.reveal_inner(row, column);
        new_universe
    }
    fn reveal_inner(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        if self.stats[idx] == CellState::Hidden {
            self.stats[idx] = CellState::Revealed;
            if self.neighbour_count(row, column) == 0 {
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
                        self.reveal_inner(neighbour_row, neighbour_col);
                    }
                }
            }
        }
    }
}

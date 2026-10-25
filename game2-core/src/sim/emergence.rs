//! Emergent effects proof: cellular automata grass/forest spread.
//! From lit review (Sweetser EmerGEnT): CA creates emergent world behavior.
//! Simple rules -> complex patterns (fire spread, forest growth).


/// Cell states for the CA world
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Empty,
    Grass,
    Tree,
    Burning,
    Burnt,
    Water,
}

/// CA world grid
#[derive(Debug)]
pub struct Automata {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<Cell>>,
    pub tick: u64,
}

impl Automata {
    pub fn new(w: usize, h: usize) -> Self {
        Self { width: w, height: h, cells: vec![vec![Cell::Empty; w]; h], tick: 0 }
    }

    /// Seed the world: grass in center, some trees
    pub fn seed(&mut self) {
        for y in 5..15 {
            for x in 5..15 {
                self.cells[y][x] = Cell::Grass;
            }
        }
        // A few trees
        self.cells[7][7] = Cell::Tree;
        self.cells[8][9] = Cell::Tree;
        self.cells[10][10] = Cell::Tree;
        self.cells[12][8] = Cell::Tree;
    }

    /// Count neighbors of a kind
    fn neighbors(&self, x: usize, y: usize, kind: Cell) -> usize {
        let mut n = 0;
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 { continue; }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < self.width && (ny as usize) < self.height {
                    if self.cells[ny as usize][nx as usize] == kind { n += 1; }
                }
            }
        }
        n
    }

    /// One CA step: grass grows to trees, trees can burn
    pub fn step(&mut self) {
        let mut next = self.cells.clone();
        for y in 0..self.height {
            for x in 0..self.width {
                let trees = self.neighbors(x, y, Cell::Tree);
                let burning = self.neighbors(x, y, Cell::Burning);
                next[y][x] = match self.cells[y][x] {
                    Cell::Empty => if trees > 0 { Cell::Grass } else { Cell::Empty },
                    Cell::Grass => if trees >= 2 { Cell::Tree } else { Cell::Grass },
                    Cell::Tree => if burning > 0 { Cell::Burning } else { Cell::Tree },
                    Cell::Burning => Cell::Burnt,
                    Cell::Burnt => if trees < 2 { Cell::Grass } else { Cell::Burnt },
                    Cell::Water => Cell::Water,
                };
            }
        }
        self.cells = next;
        self.tick += 1;
    }

    /// ASCII visualization
    pub fn ascii(&self) -> String {
        let mut s = String::new();
        for row in &self.cells {
            for c in row {
                s.push(match c {
                    Cell::Empty => '.',
                    Cell::Grass => ',',
                    Cell::Tree => 'T',
                    Cell::Burning => '!',
                    Cell::Burnt => 'x',
                    Cell::Water => '~',
                });
            }
            s.push('\n');
        }
        s
    }
}

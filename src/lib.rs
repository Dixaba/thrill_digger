use ansi_term::{Colour::*, Style};
use combinations::Combinations;
use std::{cmp::min, fmt::Display};

const WIDTH: usize = 6;
const HEIGHT: usize = 5;
const BOMBCOUNT: usize = 8;
const DEFAULT_PROB: f32 = BOMBCOUNT as f32 / (WIDTH * HEIGHT) as f32;

#[derive(Clone, Copy, PartialEq)]
pub enum Cell {
    Green,
    Blue,
    Red,
    Silver,
    Gold,
    Rupoor,
    Sus,
    Unknown(f32),
}

fn binom(n: usize, k: usize) -> usize {
    if n < k {
        return 0;
    }
    let mut res = 1;
    for i in 0..k {
        res = (res * (n - i)) / (i + 1);
    }
    res
}

impl Default for Cell {
    fn default() -> Self {
        Self::Unknown(DEFAULT_PROB)
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Cell::Green => Green.paint(format!("{:^5}", "G")),
                Cell::Blue => Blue.paint(format!("{:^5}", "B")),
                Cell::Red => Red.paint(format!("{:^5}", "R")),
                Cell::Silver => Style::default().bold().paint(format!("{:^5}", "S")),
                Cell::Gold => Yellow.paint(format!("{:^5}", "!")),
                Cell::Rupoor => Purple.paint(format!("{:^5}", "*")),
                Cell::Sus => Red.reverse().paint(format!("{:^5}", "1.0")),
                Cell::Unknown(value) => {
                    Style::default().paint(format!("{:^5.2}", value))
                }
            }
        )?;
        Ok(())
    }
}

impl Into<bool> for Cell {
    fn into(self) -> bool {
        self != Cell::Rupoor
    }
}

pub struct Field {
    cells: [[Cell; WIDTH]; HEIGHT],
    untouched_cells: Vec<usize>,
    touched_cells: Vec<usize>,
    confirmed_cells: Vec<usize>,
}

impl Default for Field {
    fn default() -> Self {
        Field {
            cells: [[Cell::default(); WIDTH]; HEIGHT],
            untouched_cells: (0..HEIGHT * WIDTH).collect(),
            touched_cells: Vec::new(),
            confirmed_cells: Vec::new(),
        }
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " |",)?;
        for cell_num in 0..WIDTH {
            write!(f, "{:^5}|", cell_num)?;
        }
        write!(f, "\n")?;

        write!(f, "-|",)?;
        for _cell_num in 0..WIDTH {
            write!(f, "{:-^5}|", '-')?;
        }
        write!(f, "\n")?;

        let mut min = 1.1;

        for row_num in 0..HEIGHT {
            for cell_num in 0..WIDTH {
                if let Cell::Unknown(value) = self.cells[row_num][cell_num] {
                    if value < min {
                        min = value;
                    }
                }
            }
        }

        for row_num in 0..HEIGHT {
            write!(f, "{}|", row_num)?;
            for cell_num in 0..WIDTH {
                write!(f, "{}|", {
                    let mut res = format!("{}", self.cells[row_num][cell_num]);
                    if let Cell::Unknown(value) = self.cells[row_num][cell_num] {
                        if value == min {
                            res = Green
                                .reverse()
                                .paint(format!("{}", self.cells[row_num][cell_num]))
                                .to_string();
                        }
                    }
                    res
                })?;
            }
            write!(f, "\n")?;
        }

        writeln!(f, "Bombs + rupoors remain: {}", self.bombs_remain())?;
        Ok(())
    }
}

impl Field {
    pub fn new() -> Self {
        Field::default()
    }

    fn matches_preset(&self, positions: &Vec<usize>) -> bool {
        for row_num in 0..HEIGHT {
            for cell_num in 0..WIDTH {
                let index = row_num * WIDTH + cell_num;
                match self.cells[row_num][cell_num] {
                    Cell::Unknown(_) => {}
                    Cell::Rupoor | Cell::Sus => {
                        if !positions.contains(&index) {
                            return false;
                        }
                    }
                    x => {
                        if x != match Field::bombs_nearby_pos(positions, index) {
                            0 => Cell::Green,
                            1 | 2 => Cell::Blue,
                            3 | 4 => Cell::Red,
                            5 | 6 => Cell::Silver,
                            7 | 8 => Cell::Gold,
                            _ => Cell::Unknown(0.0),
                        } {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    pub fn set_cell(&mut self, x: i32, y: i32, value: Cell) {
        if x < 0 || y < 0 {
            return;
        }

        let mut index;

        match self.cells[y as usize][x as usize] {
            Cell::Unknown(_) | Cell::Sus => {
                if value == Cell::Rupoor {
                    index = (y * WIDTH as i32 + x) as usize;
                    self.confirmed_cells.push(index);
                    self.untouched_cells.retain(|v| *v != index);
                    self.touched_cells.retain(|v| *v != index);
                } else {
                    for y_d in -1..=1 {
                        if y + y_d < 0 || y + y_d == HEIGHT as i32 {
                            continue;
                        }
                        for x_d in -1..=1 {
                            if x + x_d < 0 || x + x_d == WIDTH as i32 {
                                continue;
                            }
                            index = (y + y_d) as usize * WIDTH + (x + x_d) as usize;
                            self.untouched_cells.retain(|v| *v != index);
                            self.touched_cells.push(index);
                        }
                    }
                }
            }
            _ => {
                for y_d in -1..=1 {
                    if y + y_d < 0 || y + y_d == HEIGHT as i32 {
                        continue;
                    }
                    for x_d in -1..=1 {
                        if x + x_d < 0 || x + x_d == WIDTH as i32 {
                            continue;
                        }
                        index = (y + y_d) as usize * WIDTH + (x + x_d) as usize;
                        self.untouched_cells.retain(|v| *v != index);
                        self.touched_cells.push(index);
                        match self.cells[(y + y_d) as usize][(x + x_d) as usize] {
                            Cell::Sus => {
                                self.cells[(y + y_d) as usize][(x + x_d) as usize] =
                                    Cell::Unknown(0.0)
                            }
                            _ => {}
                        }
                    }
                }

                self.confirmed_cells.clear();
            }
        }

        if value == Cell::Rupoor {
            index = (y * WIDTH as i32 + x) as usize;
            self.confirmed_cells.push(index);
            self.untouched_cells.retain(|v| *v != index);
            self.touched_cells.retain(|v| *v != index);
        }

        self.cells[y as usize][x as usize] = value;
        for index in 0..WIDTH * HEIGHT {
            match self.cells[index / WIDTH][index % WIDTH] {
                Cell::Sus if !self.confirmed_cells.contains(&index) => {
                    self.cells[index / WIDTH][index % WIDTH] = Cell::Unknown(0.0)
                }
                Cell::Rupoor => {
                    self.confirmed_cells.push(index);
                }
                _ => {}
            }
        }

        self.confirmed_cells.sort_unstable();
        self.confirmed_cells.dedup();

        for row_num in 0..HEIGHT {
            for cell_num in 0..WIDTH {
                match self.cells[row_num][cell_num] {
                    Cell::Unknown(_) => {}
                    _ => {
                        index = row_num * WIDTH + cell_num;
                        self.touched_cells.retain(|v| *v != index);
                    }
                };
            }
        }

        self.touched_cells.sort_unstable();
        self.touched_cells.dedup();

        let remaining_prob = self.bombs_remain() as f32;
        let mut total_touched_prob = 0.0;

        let mut full_list;

        let max_var = min(self.bombs_remain(), self.touched_cells.len());

        let mut possible_list = [0; HEIGHT * WIDTH];
        let mut possible_list_len = 0;

        for mmm in 0..=max_var {
            let optimization_mul = binom(self.untouched_cells.len(), max_var - mmm);

            if mmm == 0 {
                // 0 bombs case
                if self.matches_preset(&self.confirmed_cells) {
                    possible_list_len += optimization_mul;
                }
            } else if mmm == self.touched_cells.len() {
                // all touched cells
                full_list = self.confirmed_cells.clone();
                full_list.append(&mut self.touched_cells.clone());
                if self.matches_preset(&full_list) {
                    for i in &self.touched_cells {
                        possible_list[*i] += optimization_mul;
                    }
                    possible_list_len += optimization_mul;
                }
            } else {
                // somewhere in between 0 and all
                for variant in Combinations::new(self.touched_cells.clone(), mmm) {
                    full_list = self.confirmed_cells.clone();
                    full_list.append(&mut variant.clone());
                    if self.matches_preset(&full_list) {
                        for i in variant {
                            possible_list[i] += optimization_mul;
                        }
                        possible_list_len += optimization_mul;
                    }
                }
            }
        }

        let fields = possible_list_len as f32;
        println!("Found {} possible fields", fields);

        for index in &self.touched_cells {
            let has = possible_list[*index] as f32;
            let prob = has / fields;
            self.cells[index / WIDTH][index % WIDTH] = if prob == 1.0 {
                self.confirmed_cells.push(*index);
                Cell::Sus
            } else {
                Cell::Unknown(prob)
            };

            total_touched_prob += prob;
        }
        // let mut fvec = self.touched_cells.clone();
        // fvec.retain(|index| self.cells[index / WIDTH][index % WIDTH] != Cell::Sus);
        // self.touched_cells = fvec;

        let untouched_prob =
            (remaining_prob - total_touched_prob) / self.untouched_cells.len() as f32;
        for index in &self.untouched_cells {
            self.cells[index / WIDTH][index % WIDTH] = if untouched_prob == 1.0 {
                self.confirmed_cells.push(*index);
                Cell::Sus
            } else {
                Cell::Unknown(untouched_prob)
            };
        }
        // let mut fvec2 = self.untouched_cells.clone();
        // fvec2.retain(|index| self.cells[index / WIDTH][index % WIDTH] != Cell::Sus);
        // self.untouched_cells = fvec2;
    }

    fn bombs_remain(&self) -> usize {
        let mut res = BOMBCOUNT;
        for row in self.cells {
            for cell in row {
                if cell == Cell::Rupoor || cell == Cell::Sus {
                    res -= 1;
                }
            }
        }
        res
    }

    fn bombs_nearby_pos(positions: &Vec<usize>, pos: usize) -> u8 {
        let mut res = 0;
        let y = (pos / WIDTH) as i32;
        let x = (pos % WIDTH) as i32;
        for y_d in -1..=1 {
            if y + y_d < 0 || y + y_d == HEIGHT as i32 {
                continue;
            }
            for x_d in -1..=1 {
                if x + x_d < 0 || x + x_d == WIDTH as i32 {
                    continue;
                }
                if positions.contains(&(((y + y_d) * WIDTH as i32 + (x + x_d)) as usize)) {
                    res += 1;
                }
            }
        }
        res
    }
}

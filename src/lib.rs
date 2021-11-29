use ansi_term::{Colour::*, Style};
use combinations::Combinations;
use rayon::prelude::*;
use std::{cmp::min, fmt::Display, sync::mpsc::channel};

const WIDTH: usize = 6;
const HEIGHT: usize = 5;
const BOMBCOUNT: usize = 8;
const DEFAULT_PROB: f32 = BOMBCOUNT as f32 / (WIDTH * HEIGHT) as f32;

#[derive(Clone, Copy)]
pub enum Cell {
    Green,
    Blue,
    Red,
    Silver,
    Gold,
    Rupoor,
    Unknown(f32),
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
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
                Cell::Unknown(value) => {
                    if value == &1.0 {
                        Red.reverse().paint(format!("{:^5}", "1.0"))
                    } else {
                        Style::default().paint(format!("{:^5.2}", value))
                    }
                }
            }
        )?;
        Ok(())
    }
}

pub struct Field {
    cells: [Cell; WIDTH * HEIGHT],
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

        for cell_num in 0..HEIGHT * WIDTH {
            if let Cell::Unknown(value) = self.cells[cell_num] {
                if value < min {
                    min = value;
                }
            }
        }

        for row_num in 0..HEIGHT {
            write!(f, "{}|", row_num)?;
            for cell_num in 0..WIDTH {
                write!(f, "{}|", {
                    let mut res = format!("{}", self.cells[row_num * WIDTH + cell_num]);
                    if let Cell::Unknown(value) = self.cells[row_num * WIDTH + cell_num] {
                        if value == min {
                            res = Green
                                .reverse()
                                .paint(format!("{}", self.cells[row_num * WIDTH + cell_num]))
                                .to_string();
                        }
                    }
                    res
                })?;
            }
            write!(f, "\n")?;
        }

        writeln!(f, "Bombs + rupoors remain: {}", self.bombs_remain(true))?;
        Ok(())
    }
}

impl Field {
    pub fn new() -> Self {
        Field {
            cells: [Cell::default(); WIDTH * HEIGHT],
        }
    }

    fn matches_preset(&self, positions: &Vec<usize>) -> bool {
        for cell_num in 0..HEIGHT * WIDTH {
            match self.cells[cell_num] {
                Cell::Unknown(_) => {}
                Cell::Rupoor => {
                    if !positions.contains(&cell_num) {
                        return false;
                    }
                }
                x => {
                    if x != match Field::bombs_nearby_pos(positions, cell_num) {
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
        true
    }

    pub fn set_cell(&mut self, x: i32, y: i32, value: Cell) {
        if x < 0 || y < 0 {
            return;
        }

        if self.cells[y as usize * WIDTH + x as usize] == value {
            return;
        }
        self.cells[y as usize * WIDTH + x as usize] = value;

        let mut untouched_cells = (0..HEIGHT * WIDTH).collect::<Vec<usize>>();
        let mut touched_cells = Vec::new();
        let mut confirmed_cells = Vec::new();

        for index in 0..WIDTH * HEIGHT {
            match self.cells[index] {
                Cell::Rupoor => {
                    confirmed_cells.push(index);
                    untouched_cells.retain(|v| *v != index);
                }
                Cell::Green | Cell::Blue | Cell::Red | Cell::Silver | Cell::Gold => {
                    let local_x = (index % WIDTH) as i32;
                    let local_y = (index / WIDTH) as i32;

                    for y_d in -1..=1 {
                        if local_y + y_d < 0 || local_y + y_d == HEIGHT as i32 {
                            continue;
                        }
                        for x_d in -1..=1 {
                            if local_x + x_d < 0 || local_x + x_d == WIDTH as i32 {
                                continue;
                            }
                            let index2 =
                                (local_y + y_d) as usize * WIDTH + (local_x + x_d) as usize;
                            untouched_cells.retain(|v| *v != index2);
                            touched_cells.push(index2);
                        }
                    }
                }
                _ => {}
            }
        }

        for cell_num in 0..HEIGHT * WIDTH {
            match self.cells[cell_num] {
                Cell::Unknown(_) => {}
                _ => {
                    touched_cells.retain(|v| *v != cell_num);
                }
            };
        }

        touched_cells.sort_unstable();
        touched_cells.dedup();

        let remaining_prob = self.bombs_remain(false) as f32;
        let mut total_touched_prob = 0.0;

        let max_var = min(self.bombs_remain(false), touched_cells.len());

        let (sender, receiver) = channel();

        (0..=max_var)
            .into_par_iter()
            .for_each_with(sender, |s, mmm| {
                let optimization_mul = binom(untouched_cells.len(), max_var - mmm);

                let mut counts = [0; HEIGHT * WIDTH];

                if mmm == 0 {
                    // 0 bombs case
                    if self.matches_preset(&confirmed_cells) {
                        s.send((counts, optimization_mul)).unwrap();
                    }
                } else if mmm == touched_cells.len() {
                    // all touched cells
                    let mut full_list = confirmed_cells.clone();
                    full_list.append(&mut touched_cells.clone());
                    if self.matches_preset(&full_list) {
                        for i in &touched_cells {
                            counts[*i] += optimization_mul;
                        }
                        s.send((counts, optimization_mul)).unwrap();
                    }
                } else {
                    // somewhere in between 0 and all

                    let mut number = 0;

                    for variant in Combinations::new(touched_cells.clone(), mmm) {
                        let mut full_list = confirmed_cells.clone();
                        full_list.append(&mut variant.clone());
                        if self.matches_preset(&full_list) {
                            for i in variant {
                                counts[i] += optimization_mul;
                            }
                            number += optimization_mul;
                        }
                    }
                    s.send((counts, number)).unwrap();
                }
            });

        let mut possible_list = [0; HEIGHT * WIDTH];
        let mut possible_list_len = 0;

        receiver
            .iter()
            .for_each(|r: ([usize; HEIGHT * WIDTH], usize)| {
                for i in 0..HEIGHT * WIDTH {
                    possible_list[i] += r.0[i];
                }

                possible_list_len += r.1;
            });

        let fields = possible_list_len as f32;
        println!("Found {} possible fields", fields);

        for index in &touched_cells {
            let has = possible_list[*index] as f32;
            let prob = has / fields;
            self.cells[*index] = Cell::Unknown(prob);

            total_touched_prob += prob;
        }

        let untouched_prob = (remaining_prob - total_touched_prob) / untouched_cells.len() as f32;
        for index in &untouched_cells {
            self.cells[*index] = Cell::Unknown(untouched_prob);
        }
    }

    fn bombs_remain(&self, count_unknown: bool) -> usize {
        let mut res = BOMBCOUNT;
        for cell in self.cells {
            match cell {
                Cell::Rupoor => {
                    res -= 1;
                }
                Cell::Unknown(value) if value == 1.0 && count_unknown => {
                    res -= 1;
                }
                _ => {}
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

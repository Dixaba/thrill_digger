use ansi_term::{Colour::*, Style};
use combinations::Combinations;
use std::fmt::Display;

#[macro_use]
extern crate lazy_static;

const WIDTH: usize = 6;
const HEIGHT: usize = 5;
const BOMBCOUNT: usize = 8;
const DEFAULT_PROB: f32 = BOMBCOUNT as f32 / (WIDTH * HEIGHT) as f32;

lazy_static! {
    static ref ALL_COMBS: Vec<Vec<usize>> =
        Combinations::new((0..HEIGHT * WIDTH).collect(), BOMBCOUNT as usize).collect();
    static ref ALL_FIELDS: Vec<Field> = {
        let mut res = Vec::new();

        for current_comb in ALL_COMBS.iter() {
            res.push(Field::new_preset(current_comb));
        }

        res
    };
}

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

pub struct Field {
    cells: [[Cell; WIDTH]; HEIGHT],
    possible_fields: Vec<&'static Field>,
}

impl Default for Field {
    fn default() -> Self {
        Field {
            cells: [[Cell::default(); WIDTH]; HEIGHT],
            possible_fields: Vec::new(),
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
        let mut f = Field::default();
        f.possible_fields = ALL_FIELDS.iter().by_ref().collect();
        f
    }

    pub fn new_preset(positions: &Vec<usize>) -> Self {
        let mut result = Field::default();
        for pos in positions {
            result.cells[pos / WIDTH][pos % WIDTH] = Cell::Rupoor;
        }

        for row_num in 0..HEIGHT {
            for cell_num in 0..WIDTH {
                if let Cell::Unknown(_) = result.cells[row_num][cell_num] {
                    result.cells[row_num][cell_num] =
                        match result.bombs_nearby(cell_num as i32, row_num as i32) {
                            0 => Cell::Green,
                            1 | 2 => Cell::Blue,
                            3 | 4 => Cell::Red,
                            5 | 6 => Cell::Silver,
                            7 | 8 => Cell::Gold,
                            _ => Cell::Unknown(0.0),
                        };
                }
            }
        }

        result
    }

    pub fn set_cell(&mut self, x: usize, y: usize, value: Cell) {
        self.cells[y][x] = value;
        self.possible_fields = self
            .possible_fields
            .iter()
            .filter(|f| f.cells[y][x] == value)
            .copied()
            .collect();

        let fields = self.possible_fields.len() as f32;

        for row_num in 0..HEIGHT {
            for cell_num in 0..WIDTH {
                if let Cell::Unknown(_) = self.cells[row_num][cell_num] {
                    let has = self
                        .possible_fields
                        .iter()
                        .filter(|f| f.cells[row_num][cell_num] == Cell::Rupoor)
                        .count() as f32;

                    let prob = has / fields;

                    self.cells[row_num][cell_num] = if prob == 1.0 {
                        Cell::Sus
                    } else {
                        Cell::Unknown(prob)
                    }
                }
            }
        }
    }

    fn bombs_remain(&self) -> u8 {
        let mut res = BOMBCOUNT;
        for row in self.cells {
            for cell in row {
                if cell == Cell::Rupoor || cell == Cell::Sus {
                    res -= 1;
                }
            }
        }
        res as u8
    }

    fn bombs_nearby(&self, x: i32, y: i32) -> u8 {
        let mut res = 0;
        for y_d in -1..=1 {
            if y + y_d < 0 || y + y_d == HEIGHT as i32 {
                continue;
            }
            for x_d in -1..=1 {
                if x + x_d < 0 || x + x_d == WIDTH as i32 {
                    continue;
                }
                if self.cells[(y + y_d) as usize][(x + x_d) as usize] == Cell::Rupoor {
                    res += 1;
                }
            }
        }
        res
    }
}

use std::io;
use thrill_digger::*;

fn main() {
    ansi_term::enable_ansi_support().unwrap();

    let mut f = Field::new();

    loop {
        print!("{}", f);

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        input = input.trim().to_string();

        if input.is_empty() {
            continue;
        }
        let chars: Vec<char> = input.chars().collect();

        // Quit
        if chars[0] == 'q' || chars[0] == 'Q' {
            break;
        }

        // New game
        if chars[0] == 'n' || chars[0] == 'N' {
            f = Field::new();
            continue;
        }

        // Proper command has 3 chars
        if chars.len() < 3 {
            continue;
        }
        let row = match String::from(chars[0]).parse() {
            Ok(it) => it,
            _ => continue,
        };
        let column = match String::from(chars[1]).parse() {
            Ok(it) => it,
            _ => continue,
        };
        let cell = chars[2];

        f.set_cell(
            row,
            column,
            match cell {
                'G' | 'g' => Cell::Green,
                'B' | 'b' => Cell::Blue,
                'R' | 'r' => Cell::Red,
                'S' | 's' => Cell::Silver,
                '!' => Cell::Gold,
                '*' => Cell::Rupoor,
                _ => Cell::Unknown(0.0),
            },
        );
    }
}

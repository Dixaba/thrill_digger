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
        if chars[0] == 'n' || chars[0] == 'N' {
            f = Field::new();
            continue;
        }
        if chars[0] == 'q' || chars[0] == 'Q' {
            break;
        }

        if let Ok(y) = String::from(chars[0]).parse() {
            if let Ok(x) = String::from(chars[1]).parse() {
                let cell = chars[2];

                f.set_cell(
                    x,
                    y,
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
    }
}

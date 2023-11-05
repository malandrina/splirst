use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead};

use clap::Parser;

static ASCII_LOWER: [char; 26] = [
    'a', 'b', 'c', 'd', 'e',
    'f', 'g', 'h', 'i', 'j',
    'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't',
    'u', 'v', 'w', 'x', 'y',
    'z',
];

static PREFIX: &str = "x";
static DEFAULT_LINE_COUNT: usize = 1000;

#[derive(Parser,Default,Debug)]
pub struct Arguments {
    #[clap(short, long)]
    line_count: Option<usize>,
    file_path: String,
}

pub fn run(args: Arguments) -> Result<(), Box<dyn Error>> {
    let file_path = args.file_path.clone();
    let line_count = match args.line_count {
        Some(lc) => lc,
        None => DEFAULT_LINE_COUNT,
    };
    let file = File::open(file_path).unwrap();

    let lines = io::BufReader::new(file).lines();
    let mut write_buffer: Vec<String> = vec![];
    let mut prefix_first_char_idx: usize = 0;
    let mut prefix_second_char_idx: usize = 0;

    for (i, line) in lines.enumerate() {
        if let Ok(l) = line {
            write_buffer.push(l);

            if i > 0 && i % line_count == 0 {
                let mut new_filename: String = String::from("");
                new_filename.push(ASCII_LOWER[prefix_first_char_idx]);
                new_filename.push(ASCII_LOWER[prefix_second_char_idx]);
                new_filename.insert_str(0, PREFIX);

                let contents = write_buffer.join("\n");
                fs::write(new_filename, contents).unwrap();

                if prefix_second_char_idx == ASCII_LOWER.len() {
                    prefix_first_char_idx += 1;
                }

                prefix_second_char_idx += 1;
                write_buffer = vec![];
            }

        }
    }

    if write_buffer.len() > 0 {
        let mut new_filename: String = String::from("");
        new_filename.push(ASCII_LOWER[prefix_first_char_idx]);
        new_filename.push(ASCII_LOWER[prefix_second_char_idx]);
        new_filename.insert_str(0, PREFIX);

        let contents = write_buffer.join("\n");
        fs::write(new_filename, contents).unwrap();
    }
    Ok(())
}

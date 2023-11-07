use regex::Regex;
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
    #[clap(short, long, group="method")]
    line_count: Option<usize>,
    #[clap(short='n', long, group="method")]
    chunk_count: Option<usize>,
    #[clap(short, long, group="method")]
    byte_count: Option<String>,
    #[clap(short, long, group="method")]
    pattern: Option<String>,
    file_path: String,
}

fn split_by_byte_count(byte_count: String, file: File) -> Result<(), Box<dyn Error>> {
    let byte_count = byte_count.parse::<usize>().unwrap();
    let mut buf_reader = io::BufReader::with_capacity(byte_count, file);

    let mut prefix_first_char_idx: usize = 0;
    let mut prefix_second_char_idx: usize = 0;

    loop {
        let length = {
            let write_buffer = buf_reader.fill_buf()?;
            if write_buffer.len() > 0 {
                let mut new_filename: String = String::from("");
                new_filename.push(ASCII_LOWER[prefix_first_char_idx]);
                new_filename.push(ASCII_LOWER[prefix_second_char_idx]);
                new_filename.insert_str(0, PREFIX);

                fs::write(new_filename, write_buffer).unwrap();

                if prefix_second_char_idx == ASCII_LOWER.len() {
                    prefix_first_char_idx += 1;
                }

                prefix_second_char_idx += 1;
            }
            write_buffer.len()
        };

        if length == 0 {
            break;
        }

        buf_reader.consume(length);
    }
    Ok(())
}

fn split_by_pattern(pattern: String, file: File) -> Result<(), Box<dyn Error>> {
    let lines = io::BufReader::new(file).lines();
    let pattern_regex = Regex::new(pattern.as_str()).unwrap();
    let mut write_buffer: Vec<String> = vec![];
    let mut prefix_first_char_idx: usize = 0;
    let mut prefix_second_char_idx: usize = 0;

    for (_ , line) in lines.enumerate() {
        if let Ok(l) = line {
            let line_matches_pattern = pattern_regex.is_match(&l);

            if line_matches_pattern && write_buffer.len() > 0 {
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

            write_buffer.push(l);
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

fn split_by_chunk_count(chunk_count: usize, file: File) -> Result<(), Box<dyn Error>> {
    let file_size = file.metadata().unwrap().len();
    let chunk_size = (file_size / chunk_count as u64) as usize;
    let first_n_chunks_size = chunk_size * (chunk_count - 1);
    let last_chunk_size = (file_size - first_n_chunks_size as u64) as usize;
    let mut buf_reader = io::BufReader::with_capacity(last_chunk_size, file);

    let mut counter = 0;
    let mut prefix_first_char_idx: usize = 0;
    let mut prefix_second_char_idx: usize = 0;

    loop {
        let length = {
            let write_buffer = buf_reader.fill_buf()?;
            counter += 1;
            if write_buffer.len() > 0 {
                let mut new_filename: String = String::from("");
                new_filename.push(ASCII_LOWER[prefix_first_char_idx]);
                new_filename.push(ASCII_LOWER[prefix_second_char_idx]);
                new_filename.insert_str(0, PREFIX);

                fs::write(new_filename, write_buffer).unwrap();

                if prefix_second_char_idx == ASCII_LOWER.len() {
                    prefix_first_char_idx += 1;
                }

                prefix_second_char_idx += 1;
            }
            write_buffer.len()
        };

        if length == 0 {
            break;
        }

        if counter == (chunk_count - 1) {
            buf_reader.consume(last_chunk_size);
        } else {
            buf_reader.consume(length);
        }
    }
    Ok(())
}

fn split_by_line_count(line_count: usize, file: File) -> Result<(), Box<dyn Error>> {
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

pub fn run(args: Arguments) -> Result<(), Box<dyn Error>> {
    let file_path = args.file_path.clone();
    let file = File::open(file_path).unwrap();
    let line_count = match args.line_count {
        Some(lc) => lc,
        None => DEFAULT_LINE_COUNT,
    };

    if let Some(chunk_count) = args.chunk_count {
        split_by_chunk_count(chunk_count, file)
    } else if let Some(byte_count) = args.byte_count {
        split_by_byte_count(byte_count, file)
    } else if let Some(pattern) = args.pattern {
        split_by_pattern(pattern, file)
    } else {
        split_by_line_count(line_count, file)
    }
}

use regex::Regex;
use std::collections::HashMap;
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

#[derive(Clone)]
struct ByteCountValueParser;

impl clap::builder::TypedValueParser for ByteCountValueParser {
    type Value = u64;

    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let b_to_gb_multiplier = f32::powi(10u32 as f32, -9);
        let unit_multipliers = HashMap::from([
            ("k", 1000),
            ("m", 1000000),
            ("g", b_to_gb_multiplier as u64)
        ]);

        let mut value = value.to_str().unwrap().split("").collect::<Vec<&str>>();
        value.retain(|i| i.len() > 0);
        let unit_index = value.len() - 1;
        let unit = value[unit_index];
        let case_insensitive_unit = unit.to_lowercase();
        let unit_key = &case_insensitive_unit[..];
        let multiplier = unit_multipliers[unit_key];

        value.remove(unit_index);
        let value = value.join("").parse::<u64>().unwrap();
        let byte_count = value * multiplier;

        Ok(byte_count)
    }
}

#[derive(Parser,Default,Debug)]
pub struct Arguments {
    #[clap(short, long, default_value="1000", group="method")]
    line_count: usize,
    #[clap(short='n', long, group="method")]
    chunk_count: Option<usize>,
    #[clap(short, long, group="method", value_parser=ByteCountValueParser)]
    byte_count: Option<u64>,
    #[clap(short, long, group="method")]
    pattern: Option<String>,
    file_path: String,
    #[clap(default_value="x")]
    prefix: String,
}

fn split_by_byte_count(byte_count: u64, file: File, prefix: String) -> Result<(), Box<dyn Error>> {
    let mut buf_reader = io::BufReader::with_capacity(byte_count as usize, file);

    let mut prefix_first_char_idx: usize = 0;
    let mut prefix_second_char_idx: usize = 0;

    loop {
        let length = {
            let write_buffer = buf_reader.fill_buf()?;
            if write_buffer.len() > 0 {
                let mut new_filename: String = String::from("");
                new_filename.push(ASCII_LOWER[prefix_first_char_idx]);
                new_filename.push(ASCII_LOWER[prefix_second_char_idx]);
                new_filename.insert_str(0, &prefix[..]);

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

fn split_by_pattern(pattern: String, file: File, prefix: String) -> Result<(), Box<dyn Error>> {
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
                new_filename.insert_str(0, &prefix[..]);

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
        new_filename.insert_str(0, &prefix[..]);

        let contents = write_buffer.join("\n");
        fs::write(new_filename, contents).unwrap();
    }
    Ok(())
}

fn split_by_chunk_count(chunk_count: usize, file: File, prefix: String) -> Result<(), Box<dyn Error>> {
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
                new_filename.insert_str(0, &prefix[..]);

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

fn split_by_line_count(line_count: usize, file: File, prefix: String) -> Result<(), Box<dyn Error>> {
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
                new_filename.insert_str(0, &prefix[..]);

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
        new_filename.insert_str(0, &prefix[..]);

        let contents = write_buffer.join("\n");
        fs::write(new_filename, contents).unwrap();
    }
    Ok(())
}

pub fn run(args: Arguments) -> Result<(), Box<dyn Error>> {
    let file_path = args.file_path.clone();
    let file = File::open(file_path).unwrap();
    let prefix = args.prefix;

    if let Some(chunk_count) = args.chunk_count {
        split_by_chunk_count(chunk_count, file, prefix)
    } else if let Some(byte_count) = args.byte_count {
        split_by_byte_count(byte_count, file, prefix)
    } else if let Some(pattern) = args.pattern {
        split_by_pattern(pattern, file, prefix)
    } else {
        split_by_line_count(args.line_count, file, prefix)
    }
}

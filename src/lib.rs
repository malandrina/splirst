use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead};

use clap::Parser;

static DEFAULT_SUFFIX_LENGTH: usize = 2;

static ASCII_LOWER: [char; 26] = [
    'a', 'b', 'c', 'd', 'e',
    'f', 'g', 'h', 'i', 'j',
    'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't',
    'u', 'v', 'w', 'x', 'y',
    'z',
];

struct FileOptions {
    file: File,
    prefix: String,
    suffix_length: usize,
    numeric_suffix: bool,
}

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
    #[clap(short='a', long, default_value="2")]
    suffix_length: usize,
    #[clap(short='d', long)]
    numeric_suffix: bool,
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

fn suffix(file_number: usize, suffix_length: usize, numeric_suffix: bool) -> String {
    let mut suffix: String = String::from("");

    if numeric_suffix {
        let n = &(file_number - 1).to_string()[..];
        suffix.insert_str(0, n);
        for _ in 0..(suffix_length - n.len()) {
            suffix.insert_str(0, "0");
        }

    } else {
      let first_char_idx = if file_number % ASCII_LOWER.len() == 0 {
          ((file_number / ASCII_LOWER.len())) - 1 as usize
      } else {
          ((file_number / ASCII_LOWER.len()) as f32).floor() as usize
      };

      let second_char_idx = (file_number - (first_char_idx * ASCII_LOWER.len())) - 1;
      let first_char = ASCII_LOWER[first_char_idx];
      let second_char = ASCII_LOWER[second_char_idx];

      for _ in 0..(suffix_length - DEFAULT_SUFFIX_LENGTH) {
          suffix.push('a')
      }

      suffix.push(first_char);
      suffix.push(second_char);
    }

    suffix
}

fn split_by_byte_count(byte_count: u64, file_options: FileOptions) -> Result<(), Box<dyn Error>> {
    let suffix_length = file_options.suffix_length;
    let numeric_suffix = file_options.numeric_suffix;
    let prefix = file_options.prefix;
    let file = file_options.file;
    let mut buf_reader = io::BufReader::with_capacity(byte_count as usize, file);
    let mut counter = 0;

    loop {
        let length = {
            let write_buffer = buf_reader.fill_buf()?;
            if write_buffer.len() > 0 {
                counter += 1;
                let mut new_filename: String = String::from("");
                let suffix = suffix(counter, suffix_length, numeric_suffix);
                new_filename.insert_str(0, &suffix[..]);
                new_filename.insert_str(0, &prefix[..]);

                fs::write(new_filename, write_buffer).unwrap();
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

fn split_by_pattern(pattern: String, file_options: FileOptions) -> Result<(), Box<dyn Error>> {
    let numeric_suffix = file_options.numeric_suffix;
    let suffix_length = file_options.suffix_length;
    let prefix = file_options.prefix;
    let file = file_options.file;
    let lines = io::BufReader::new(file).lines();
    let pattern_regex = Regex::new(pattern.as_str()).unwrap();
    let mut write_buffer: Vec<String> = vec![];
    let mut counter = 0;

    for (_ , line) in lines.enumerate() {
        if let Ok(l) = line {
            let line_matches_pattern = pattern_regex.is_match(&l);

            if line_matches_pattern && write_buffer.len() > 0 {
                counter += 1;
                let mut new_filename: String = String::from("");
                let suffix = suffix(counter, suffix_length, numeric_suffix);
                new_filename.insert_str(0, &suffix[..]);
                new_filename.insert_str(0, &prefix[..]);

                let contents = write_buffer.join("\n");
                fs::write(new_filename, contents).unwrap();
                write_buffer = vec![];
            }

            write_buffer.push(l);
        }
    }

    if write_buffer.len() > 0 {
        counter += 1;
        let mut new_filename: String = String::from("");
        let suffix = suffix(counter, suffix_length, numeric_suffix);
        new_filename.insert_str(0, &suffix[..]);
        new_filename.insert_str(0, &prefix[..]);

        let contents = write_buffer.join("\n");
        fs::write(new_filename, contents).unwrap();
    }
    Ok(())
}

fn split_by_chunk_count(chunk_count: usize, file_options: FileOptions) -> Result<(), Box<dyn Error>> {
    let suffix_length = file_options.suffix_length;
    let numeric_suffix = file_options.numeric_suffix;
    let prefix = file_options.prefix;
    let file = file_options.file;
    let file_size = file.metadata().unwrap().len();
    let chunk_size = (file_size / chunk_count as u64) as usize;
    let first_n_chunks_size = chunk_size * (chunk_count - 1);
    let last_chunk_size = (file_size - first_n_chunks_size as u64) as usize;
    let mut buf_reader = io::BufReader::with_capacity(last_chunk_size, file);

    let mut counter = 0;

    loop {
        let length = {
            let write_buffer = buf_reader.fill_buf()?;
            counter += 1;
            if write_buffer.len() > 0 {
                let mut new_filename: String = String::from("");
                let suffix = suffix(counter, suffix_length, numeric_suffix);
                new_filename.insert_str(0, &suffix[..]);
                new_filename.insert_str(0, &prefix[..]);

                fs::write(new_filename, write_buffer).unwrap();
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

fn split_by_line_count(line_count: usize, file_options: FileOptions) -> Result<(), Box<dyn Error>> {
    let suffix_length = file_options.suffix_length;
    let numeric_suffix = file_options.numeric_suffix;
    let prefix = file_options.prefix;
    let file = file_options.file;
    let lines = io::BufReader::new(file).lines();
    let mut write_buffer: Vec<String> = vec![];
    let mut counter = 0;

    for (i, line) in lines.enumerate() {
        if let Ok(l) = line {
            write_buffer.push(l);

            if i > 0 && i % line_count == 0 {
                counter += 1;
                let mut new_filename: String = String::from("");
                let suffix = suffix(counter, suffix_length, numeric_suffix);
                new_filename.insert_str(0, &suffix[..]);
                new_filename.insert_str(0, &prefix[..]);

                let contents = write_buffer.join("\n");
                fs::write(new_filename, contents).unwrap();
                write_buffer = vec![];
            }
        }
    }

    if write_buffer.len() > 0 {
        counter += 1;
        let mut new_filename: String = String::from("");
        let suffix = suffix(counter, suffix_length, numeric_suffix);
        new_filename.insert_str(0, &suffix[..]);
        new_filename.insert_str(0, &prefix[..]);

        let contents = write_buffer.join("\n");
        fs::write(new_filename, contents).unwrap();
    }
    Ok(())
}

pub fn run(args: Arguments) -> Result<(), Box<dyn Error>> {
    let file_path = args.file_path.clone();
    let file = File::open(file_path).unwrap();
    let suffix_length = args.suffix_length;
    let numeric_suffix = args.numeric_suffix;
    let prefix = args.prefix;
    let file_options = FileOptions { file, prefix, suffix_length, numeric_suffix };

    if let Some(chunk_count) = args.chunk_count {
        split_by_chunk_count(chunk_count, file_options)
    } else if let Some(byte_count) = args.byte_count {
        split_by_byte_count(byte_count, file_options)
    } else if let Some(pattern) = args.pattern {
        split_by_pattern(pattern, file_options)
    } else {
        split_by_line_count(args.line_count, file_options)
    }
}

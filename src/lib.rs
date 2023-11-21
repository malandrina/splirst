use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{File};
use std::io::{self, BufRead, Write};

use clap::Parser;

static ASCII_LOWER: [char; 26] = [
    'a', 'b', 'c', 'd', 'e',
    'f', 'g', 'h', 'i', 'j',
    'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't',
    'u', 'v', 'w', 'x', 'y',
    'z',
];
static DEFAULT_SUFFIX_FIRST_CHAR: char = 'a';
static DEFAULT_SUFFIX_LENGTH: usize = 2;

struct Filename;

impl Filename {
    pub fn build(file_number: usize, suffix_length: usize, numeric_suffix: bool, prefix: String) -> String {
        let mut filename: String = String::from("");
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
              suffix.push(DEFAULT_SUFFIX_FIRST_CHAR)
          }

          suffix.push(first_char);
          suffix.push(second_char);
        }

        filename.insert_str(0, &suffix[..]);
        filename.insert_str(0, &prefix[..]);
        filename
    }
}

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
        let b_to_gb_multiplier = (1.0/f32::powi(10u32 as f32, -9)) as u64;
        let unit_multipliers = HashMap::from([
            ("k", 1000),
            ("m", 1000000),
            ("g", b_to_gb_multiplier)
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
    #[clap(short='a', long, default_value="2", value_parser=2..=13)]
    suffix_length: i64,
    #[clap(short='d', long)]
    numeric_suffix: bool,
    #[clap(short, long, default_value="1000", group="method")]
    line_count: usize,
    #[clap(short='n', long, group="method", value_parser=0..=676)]
    chunk_count: Option<i64>,
    #[clap(short, long, group="method", value_parser=ByteCountValueParser)]
    byte_count: Option<u64>,
    #[clap(short, long, group="method")]
    pattern: Option<String>,
    file_path: String,
    #[clap(default_value="x")]
    prefix: String,
}

fn split_by_byte_count(byte_count: u64, file_options: FileOptions) -> Result<(), Box<dyn Error>> {
    let FileOptions { numeric_suffix, suffix_length, prefix, file } = file_options;
    let mut buf_reader = io::BufReader::with_capacity(byte_count as usize, file);
    let mut counter = 0;

    loop {
        let length = {
            let write_buffer = buf_reader.fill_buf()?;
            if write_buffer.len() > 0 {
                counter += 1;
                let new_filename = Filename::build(counter, suffix_length, numeric_suffix, prefix.clone());
                let mut new_file = File::create(new_filename).unwrap();
                new_file.write_all(write_buffer).unwrap();
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
    let FileOptions { numeric_suffix, suffix_length, prefix, file } = file_options;
    let lines = io::BufReader::new(file).lines();
    let pattern_regex = Regex::new(pattern.as_str()).unwrap();
    let mut write_buffer: Vec<String> = vec![];
    let mut counter = 0;

    for (_ , line) in lines.enumerate() {
        if let Ok(l) = line {
            let line_matches_pattern = pattern_regex.is_match(&l);

            if line_matches_pattern && write_buffer.len() > 0 {
                counter += 1;
                let new_filename = Filename::build(counter, suffix_length, numeric_suffix, prefix.clone());
                let contents = write_buffer.join("\n");
                let mut new_file = File::create(new_filename).unwrap();
                new_file.write_all(contents.as_bytes()).unwrap();
                write_buffer = vec![];
            }

            write_buffer.push(l);
        }
    }

    if write_buffer.len() > 0 {
        counter += 1;
        let new_filename = Filename::build(counter, suffix_length, numeric_suffix, prefix);
        let contents = write_buffer.join("\n");
        let mut new_file = File::create(new_filename).unwrap();
        new_file.write_all(contents.as_bytes()).unwrap();
    }
    Ok(())
}

fn split_by_chunk_count(chunk_count: usize, file_options: FileOptions) -> Result<(), Box<dyn Error>> {
    let FileOptions { numeric_suffix, suffix_length, prefix, file } = file_options;
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
                let new_filename = Filename::build(counter, suffix_length, numeric_suffix, prefix.clone());
                let mut new_file = File::create(new_filename).unwrap();
                new_file.write_all(write_buffer).unwrap();
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
    let FileOptions { numeric_suffix, suffix_length, prefix, file } = file_options;
    let lines = io::BufReader::new(file).lines();
    let mut write_buffer: Vec<String> = vec![];
    let mut counter = 0;

    for (i, line) in lines.enumerate() {
        if let Ok(l) = line {
            write_buffer.push(l);

            if i > 0 && i % line_count == 0 {
                counter += 1;
                let new_filename = Filename::build(counter, suffix_length, numeric_suffix, prefix.clone());
                let contents = write_buffer.join("\n");
                let mut new_file = File::create(new_filename).unwrap();
                new_file.write_all(contents.as_bytes()).unwrap();
                write_buffer = vec![];
            }
        }
    }

    if write_buffer.len() > 0 {
        counter += 1;
        let new_filename = Filename::build(counter, suffix_length, numeric_suffix, prefix);
        let contents = write_buffer.join("\n");
        let mut new_file = File::create(new_filename).unwrap();
        new_file.write_all(contents.as_bytes()).unwrap();
    }
    Ok(())
}

pub fn run(args: Arguments) -> Result<(), Box<dyn Error>> {
    let file_path = args.file_path.clone();
    let file = File::open(file_path).unwrap();
    let Arguments { numeric_suffix, suffix_length, prefix, .. } = args;
    let suffix_length = suffix_length as usize;
    let file_options = FileOptions { file, prefix, suffix_length, numeric_suffix };

    if let Some(chunk_count) = args.chunk_count {
        let chunk_count = chunk_count as usize;
        split_by_chunk_count(chunk_count, file_options)
    } else if let Some(byte_count) = args.byte_count {
        split_by_byte_count(byte_count, file_options)
    } else if let Some(pattern) = args.pattern {
        split_by_pattern(pattern, file_options)
    } else {
        split_by_line_count(args.line_count, file_options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn split_file_by_chunk_count() {
        let file = File::open("./src/test/fixtures/war-and-peace-excerpt.txt").unwrap();
        let file_size = file.metadata().unwrap().len() as usize;
        let prefix = String::from("a");
        let suffix_length = 2;
        let numeric_suffix = false;
        let chunk_count = 2;
        let file_options = FileOptions { file, prefix, suffix_length, numeric_suffix };

        let _ = split_by_chunk_count(chunk_count, file_options);

        let expected_file_1 = File::open("aaa").unwrap();
        let expected_file_2 = File::open("aab").unwrap();
        let expected_file_1_size = expected_file_1.metadata().unwrap().len().try_into().unwrap();
        let expected_file_2_size = expected_file_2.metadata().unwrap().len().try_into().unwrap();

        let result = std::panic::catch_unwind(|| {
            assert_eq!(file_size/chunk_count, expected_file_1_size);
            assert_eq!(file_size/chunk_count, expected_file_2_size)
        });

        fs::remove_file("aaa").unwrap();
        fs::remove_file("aab").unwrap();

        assert!(result.is_ok());
    }

    #[test]
    fn split_file_by_line_count() {
        let file = File::open("./src/test/fixtures/war-and-peace-excerpt.txt").unwrap();
        let prefix = String::from("b");
        let suffix_length = 2;
        let numeric_suffix = false;
        let line_count = 546;
        let file_options = FileOptions { file, prefix, suffix_length, numeric_suffix };

        let _ = split_by_line_count(line_count, file_options);

        let expected_file_1 = File::open("baa").unwrap();
        let expected_file_2 = File::open("bab").unwrap();
        let expected_file_1_size = expected_file_1.metadata().unwrap().len().try_into().unwrap();
        let expected_file_2_size = expected_file_2.metadata().unwrap().len().try_into().unwrap();

        let result = std::panic::catch_unwind(|| {
            assert_eq!(64137, expected_file_1_size);
            assert_eq!(58773, expected_file_2_size);
        });

        fs::remove_file("baa").unwrap();
        fs::remove_file("bab").unwrap();

        assert!(result.is_ok());
    }

    #[test]
    fn split_file_by_byte_count() {
        let file = File::open("./src/test/fixtures/war-and-peace-excerpt.txt").unwrap();
        let file_size = file.metadata().unwrap().len() as usize;
        let prefix = String::from("c");
        let suffix_length = 2;
        let numeric_suffix = false;
        let byte_count = 100000;
        let file_options = FileOptions { file, prefix, suffix_length, numeric_suffix };
        let expected_file_1_size = byte_count;
        let expected_file_2_size = file_size - byte_count;

        let _ = split_by_byte_count(byte_count as u64, file_options);

        let result_file_1 = File::open("caa").unwrap();
        let result_file_2 = File::open("cab").unwrap();
        let result_file_1_size = result_file_1.metadata().unwrap().len().try_into().unwrap();
        let result_file_2_size = result_file_2.metadata().unwrap().len().try_into().unwrap();

        let result = std::panic::catch_unwind(|| {
            assert_eq!(expected_file_1_size, result_file_1_size);
            assert_eq!(expected_file_2_size, result_file_2_size);
        });

        fs::remove_file("caa").unwrap();
        fs::remove_file("cab").unwrap();

        assert!(result.is_ok());
    }

    #[test]
    fn split_file_by_pattern() -> () {
        let file = File::open("./src/test/fixtures/war-and-peace-excerpt.txt").unwrap();
        let prefix = String::from("d");
        let suffix_length = 2;
        let numeric_suffix = false;
        let pattern = String::from("Lucca");
        let file_options = FileOptions { file, prefix, suffix_length, numeric_suffix };
        let expected_file_1_size = 35999;
        let expected_file_2_size = 86911;

        let _ = split_by_pattern(pattern, file_options);

        let result_file_1 = File::open("daa").unwrap();
        let result_file_2 = File::open("dab").unwrap();
        let result_file_1_size = result_file_1.metadata().unwrap().len().try_into().unwrap();
        let result_file_2_size = result_file_2.metadata().unwrap().len().try_into().unwrap();

        let result = std::panic::catch_unwind(|| {
            assert_eq!(expected_file_1_size, result_file_1_size);
            assert_eq!(expected_file_2_size, result_file_2_size)
        });

        fs::remove_file("daa").unwrap();
        fs::remove_file("dab").unwrap();

        assert!(result.is_ok());
    }
}

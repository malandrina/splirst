use std::fs::{File};
use splirst;

#[test]
fn split_by_chunk_count() {
    let file = File::open("./tests/fixtures/war-and-peace-excerpt.txt").unwrap();
    let file_size = file.metadata().unwrap().len() as usize;
    let prefix = String::from("a");
    let chunk_count = 2;
    let args = splirst::Arguments {
        suffix_length: 2,
        numeric_suffix: false,
        prefix,
        file_path: String::from("./tests/fixtures/war-and-peace-excerpt.txt"),
        chunk_count: Some(chunk_count),
        byte_count: None,
        line_count: 1000,
        pattern: None,
    };

    let _ = splirst::run(args);

    let expected_file_1 = File::open("aaa").unwrap();
    let expected_file_2 = File::open("aab").unwrap();
    let expected_file_1_size = expected_file_1.metadata().unwrap().len().try_into().unwrap();
    let expected_file_2_size = expected_file_2.metadata().unwrap().len().try_into().unwrap();

    let result = std::panic::catch_unwind(|| {
        assert_eq!(file_size/chunk_count as usize, expected_file_1_size);
        assert_eq!(file_size/chunk_count as usize, expected_file_2_size)
    });

    std::fs::remove_file("aaa").unwrap();
    std::fs::remove_file("aab").unwrap();

    assert!(result.is_ok());
}

#[test]
fn split_by_line_count() {
    let prefix = String::from("b");
    let args = splirst::Arguments {
        suffix_length: 2,
        numeric_suffix: false,
        prefix,
        file_path: String::from("./tests/fixtures/war-and-peace-excerpt.txt"),
        chunk_count: None,
        byte_count: None,
        line_count: 546,
        pattern: None,
    };

    let _ = splirst::run(args);

    let expected_file_1 = File::open("baa").unwrap();
    let expected_file_2 = File::open("bab").unwrap();
    let expected_file_1_size = expected_file_1.metadata().unwrap().len().try_into().unwrap();
    let expected_file_2_size = expected_file_2.metadata().unwrap().len().try_into().unwrap();

    let result = std::panic::catch_unwind(|| {
        assert_eq!(64137, expected_file_1_size);
        assert_eq!(58773, expected_file_2_size);
    });

    std::fs::remove_file("baa").unwrap();
    std::fs::remove_file("bab").unwrap();

    assert!(result.is_ok());
}

#[test]
fn split_by_byte_count() {
    let prefix = String::from("c");
    let args = splirst::Arguments {
        suffix_length: 2,
        numeric_suffix: false,
        prefix,
        file_path: String::from("./tests/fixtures/war-and-peace-excerpt.txt"),
        chunk_count: None,
        byte_count: Some(100000),
        line_count: 1000,
        pattern: None,
    };

    let _ = splirst::run(args);

    let result_file_1 = File::open("caa").unwrap();
    let result_file_2 = File::open("cab").unwrap();
    let result_file_1_size = result_file_1.metadata().unwrap().len().try_into().unwrap();
    let result_file_2_size = result_file_2.metadata().unwrap().len().try_into().unwrap();

    let result = std::panic::catch_unwind(|| {
        assert_eq!(100000, result_file_1_size);
        assert_eq!(22912, result_file_2_size);
    });

    std::fs::remove_file("caa").unwrap();
    std::fs::remove_file("cab").unwrap();

    assert!(result.is_ok());
}

#[test]
fn split_by_pattern() -> () {
    let prefix = String::from("d");
    let pattern = Some(String::from("Lucca"));
    let args = splirst::Arguments {
        suffix_length: 2,
        numeric_suffix: false,
        prefix,
        file_path: String::from("./tests/fixtures/war-and-peace-excerpt.txt"),
        chunk_count: None,
        byte_count: None,
        line_count: 1000,
        pattern,
    };
    let expected_file_1_size = 35999;
    let expected_file_2_size = 86911;

    let _ = splirst::run(args);

    let result_file_1 = File::open("daa").unwrap();
    let result_file_2 = File::open("dab").unwrap();
    let result_file_1_size = result_file_1.metadata().unwrap().len().try_into().unwrap();
    let result_file_2_size = result_file_2.metadata().unwrap().len().try_into().unwrap();

    let result = std::panic::catch_unwind(|| {
        assert_eq!(expected_file_1_size, result_file_1_size);
        assert_eq!(expected_file_2_size, result_file_2_size)
    });

    std::fs::remove_file("daa").unwrap();
    std::fs::remove_file("dab").unwrap();

    assert!(result.is_ok());
}

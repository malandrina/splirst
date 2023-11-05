use std::process;

use clap::Parser;

use splirst::Arguments;

fn main() {
    let args = Arguments::parse();

    if let Err(e) = splirst::run(args) {
        eprintln!("Application error: {e}");
        process::exit(1)
    }
}

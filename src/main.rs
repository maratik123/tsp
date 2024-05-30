use std::{fs, io};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use clap::Parser;
use clap_stdin::{FileOrStdin, Source};

use tsp::consts::ENTRY_LEN;
use tsp::parser::record::parse_airport_primary_records;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// The input file. With no input file, or when input file is -, read standard input
    #[clap(default_value = "-")]
    input: FileOrStdin,
    /// Output file. If omitted, write to standard output
    #[clap(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let (mut stdin_read, mut file_read);
    let readable: &mut dyn io::Read = match args.input.source {
        Source::Stdin => {
            stdin_read = io::stdin().lock();
            &mut stdin_read
        }
        Source::Arg(file) => {
            file_read = fs::File::open(file).unwrap();
            &mut file_read
        }
    };
    let mut readable = BufReader::new(readable);
    let mut buf = Vec::with_capacity(ENTRY_LEN + 1);
    while readable.read_until(b'\n', &mut buf).unwrap() != 0 {
        let rec = buf
            .iter()
            .rposition(|&c| c != b'\n')
            .map_or_else(|| &buf[..0], |i| &buf[0..=i]);
        if let Some(rec) = parse_airport_primary_records(rec) {
            println!("{:?}", rec)
        }
        buf.clear();
    }
}

use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::{fs, io};

use clap::Parser;
use clap_stdin::{FileOrStdin, Source};

use tsp::parser::record::parse_airport_primary_records;
use tsp::types::field::{LatitudeHemisphere, LongitudeHemisphere};
use tsp::util::trim_right_0d;

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
    let readable: &mut dyn Read = match args.input.source {
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
    let mut buf = vec![];
    readable.read_to_end(&mut buf).unwrap();
    let buf = &buf[..];
    buf.split(|&c| c == b'\n')
        .map(trim_right_0d)
        .filter_map(parse_airport_primary_records)
        .for_each(|rec| {
            println!(
                "{} ({}): {}{}°{}′{}.{:02}″, {}{}°{}′{}.{:02}″",
                rec.icao_identifier,
                rec.airport_name,
                match rec.airport_reference_point_longitude.hemisphere {
                    LongitudeHemisphere::East => 'E',
                    LongitudeHemisphere::West => 'W',
                },
                rec.airport_reference_point_longitude.degrees,
                rec.airport_reference_point_longitude.minutes,
                rec.airport_reference_point_longitude.seconds,
                rec.airport_reference_point_longitude.fractional_seconds,
                match rec.airport_reference_point_latitude.hemisphere {
                    LatitudeHemisphere::North => 'N',
                    LatitudeHemisphere::South => 'S',
                },
                rec.airport_reference_point_latitude.degrees,
                rec.airport_reference_point_latitude.minutes,
                rec.airport_reference_point_latitude.seconds,
                rec.airport_reference_point_latitude.fractional_seconds,
            );
        });
}

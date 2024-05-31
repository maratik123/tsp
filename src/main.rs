use std::collections::HashSet;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::{fs, io};

use clap::Parser;
use clap_stdin::{FileOrStdin, Source};
use tsp::math::great_circle;

use tsp::parser::file::parse_airport_primary_records;
use tsp::types::field::coord::{
    Coord, Latitude, LatitudeHemisphere, Longitude, LongitudeHemisphere,
};
use tsp::types::record::AirportPrimaryRecord;
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
    /// Output airport primary records
    #[clap(short, long)]
    aps: bool,
    /// Filter file
    #[clap(short, long)]
    filter: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let buf = {
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
        buf
    };
    let buf = &buf[..];

    let (hs, has_not_filter) = if let Some(filter) = args.filter {
        let mut items = vec![];
        BufReader::new(fs::File::open(filter).unwrap())
            .read_to_end(&mut items)
            .unwrap();
        let r_hs: Result<HashSet<_>, _> = items
            .split(|&c| c == b'\n')
            .map(trim_right_0d)
            .filter(|item| item.len() == 4)
            .map(Vec::from)
            .map(String::from_utf8)
            .collect();
        (r_hs.unwrap(), false)
    } else {
        (HashSet::new(), true)
    };

    let out = args.output;
    if args.aps {
        print_aps(
            parse_airport_primary_records(buf)
                .filter(|rec| has_not_filter || hs.contains(rec.icao_identifier)),
            out,
        );
    }
}

fn print_aps<'a>(recs: impl Iterator<Item = AirportPrimaryRecord<'a>>, out: Option<PathBuf>) {
    let (mut stdout_write, mut file_write);
    let writable: &mut dyn Write = if let Some(path) = out {
        file_write = fs::File::create(path).unwrap();
        &mut file_write
    } else {
        stdout_write = io::stdout().lock();
        &mut stdout_write
    };
    let mut writable = BufWriter::new(writable);
    let koak = Coord::from((
        &Latitude {
            degrees: 37,
            minutes: 43,
            seconds: 17,
            fractional_seconds: 0,
            hemisphere: LatitudeHemisphere::North,
        },
        &Longitude {
            degrees: 122,
            minutes: 13,
            seconds: 15,
            fractional_seconds: 0,
            hemisphere: LongitudeHemisphere::West,
        },
    ));
    for rec in recs {
        writeln!(
            &mut writable,
            "{} ({}): {}°{}′{}.{:02}″{}({}), {}°{}′{}.{:02}″{}({}). Distance to KOAK: {:.01}",
            rec.icao_identifier,
            rec.airport_name,
            rec.airport_reference_point_latitude.degrees,
            rec.airport_reference_point_latitude.minutes,
            rec.airport_reference_point_latitude.seconds,
            rec.airport_reference_point_latitude.fractional_seconds,
            match rec.airport_reference_point_latitude.hemisphere {
                LatitudeHemisphere::North => 'N',
                LatitudeHemisphere::South => 'S',
            },
            f64::from(&rec.airport_reference_point_latitude),
            rec.airport_reference_point_longitude.degrees,
            rec.airport_reference_point_longitude.minutes,
            rec.airport_reference_point_longitude.seconds,
            rec.airport_reference_point_longitude.fractional_seconds,
            match rec.airport_reference_point_longitude.hemisphere {
                LongitudeHemisphere::East => 'E',
                LongitudeHemisphere::West => 'W',
            },
            f64::from(&rec.airport_reference_point_longitude),
            great_circle(
                &koak,
                &(
                    &rec.airport_reference_point_latitude,
                    &rec.airport_reference_point_longitude
                )
                    .into()
            )
        )
        .unwrap();
    }
}

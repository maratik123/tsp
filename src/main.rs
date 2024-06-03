use std::collections::HashSet;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::{fs, io};

use clap::Parser;
use clap_stdin::{FileOrStdin, Source};

use tsp::aco::Aco;
use tsp::distance::DistancesIdx;
use tsp::model::{Airport, AirportIdx};
use tsp::parser::file::parse_airport_primary_records;
use tsp::types::field::coord::{LatitudeHemisphere, LongitudeHemisphere};
use tsp::types::record::AirportPrimaryRecord;
use tsp::util::{cycling, trim_0d};

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
    print_aps: bool,
    /// Filter file
    #[clap(short, long)]
    filter: Option<PathBuf>,
    /// Number of ants
    #[clap(default_value = "50", short, long)]
    ants: u32,
    /// Number of iterations
    #[clap(default_value = "100", short, long)]
    iterations: u32,
    /// Evaporation rate
    #[clap(default_value = "0.1", short, long)]
    evaporation: f64,
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

    let hs = if let Some(filter) = args.filter {
        let mut items = vec![];
        BufReader::new(fs::File::open(filter).unwrap())
            .read_to_end(&mut items)
            .unwrap();
        let r_hs: Result<HashSet<_>, _> = items
            .split(|&c| c == b'\n')
            .map(trim_0d)
            .filter(|item| item.len() == 4)
            .map(Vec::from)
            .map(String::from_utf8)
            .collect();
        Some(r_hs.unwrap())
    } else {
        None
    };

    let recs: Vec<_> = parse_airport_primary_records(buf)
        .filter(|rec| {
            hs.as_ref()
                .map_or(true, |hs| hs.contains(rec.icao_identifier))
        })
        .collect();

    let airports: Vec<_> = recs.iter().map(Airport::from).collect();
    let apt_idx = AirportIdx::new(&airports).unwrap();
    let distances = DistancesIdx::from(&apt_idx);

    let aco = Aco::new(&distances, None, None);
    let aco = aco.aco(args.iterations, args.ants, 1.0 - args.evaporation);
    println!("Selected cycle {aco:?}");

    if args.print_aps {
        print_aps(&recs, &distances, &aco, args.output);
    }
}

fn print_aps<'a: 'b, 'b>(
    recs: &'b [AirportPrimaryRecord<'a>],
    distances_idx: &DistancesIdx,
    aco: &[u32],
    out: Option<PathBuf>,
) {
    let (mut stdout_write, mut file_write);
    let writable: &mut dyn Write = if let Some(path) = out {
        file_write = fs::File::create(path).unwrap();
        &mut file_write
    } else {
        stdout_write = io::stdout().lock();
        &mut stdout_write
    };
    let mut writable = BufWriter::new(writable);

    for (i, j, rec, rec_next) in
        cycling(aco).map(|(&i, &j)| (i, j, recs[i as usize], recs[j as usize]))
    {
        let lat = &rec.airport_reference_point_latitude;
        let lon = &rec.airport_reference_point_longitude;
        writeln!(
            &mut writable,
            "{} ({}): {}°{}′{}.{:02}″{} {}°{}′{}.{:02}″{}. Distance to next {}: {:.01}",
            rec.icao_identifier,
            rec.airport_name,
            lat.degrees,
            lat.minutes,
            lat.seconds,
            lat.fractional_seconds,
            match lat.hemisphere {
                LatitudeHemisphere::North => 'N',
                LatitudeHemisphere::South => 'S',
            },
            lon.degrees,
            lon.minutes,
            lon.seconds,
            lon.fractional_seconds,
            match lon.hemisphere {
                LongitudeHemisphere::East => 'E',
                LongitudeHemisphere::West => 'W',
            },
            rec_next.icao_identifier,
            distances_idx.between(i, j).unwrap_or(f64::NAN)
        )
        .unwrap();
    }
}

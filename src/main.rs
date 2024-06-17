use ab_glyph::{FontRef, PxScale};
use clap::Parser;
use clap_stdin::{FileOrStdin, Source};
use image::buffer::ConvertBuffer;
use image::{RgbImage, Rgba, RgbaImage};
use imageproc::drawing::{
    draw_antialiased_line_segment_mut, draw_hollow_circle_mut, draw_text_mut,
};
use imageproc::pixelops::interpolate;
use std::collections::HashSet;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::{fs, io};
use tsp::aco::Aco;
use tsp::distance::DistancesIdx;
use tsp::model::{Airport, AirportIdx};
use tsp::parser::file::parse_airport_primary_records;
use tsp::scaler::Scaler;
use tsp::types::field::coord::{Coord, LatitudeHemisphere, LongitudeHemisphere};
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
    /// Evaporation rate (from 0 to 1)
    #[clap(default_value = "0.1", short, long)]
    evaporation: f64,
    /// Alpha
    #[clap(default_value = "0.9", long)]
    alpha: f64,
    /// Beta
    #[clap(default_value = "1.5", long)]
    beta: f64,
    /// Show unfiltered
    #[clap(short, long)]
    unfiltered: bool,
    /// Output images directory
    #[clap(long)]
    images: Option<PathBuf>,
    /// Minimal allowable distance
    #[clap(short, long)]
    min_dist: Option<f64>,
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
    let distances = DistancesIdx::from(&apt_idx, args.min_dist);

    let aco = Aco::new(&distances, None, None);
    let (aco, dist) = aco.aco(
        args.iterations,
        args.ants,
        1.0 - args.evaporation,
        args.alpha,
        args.beta,
    );
    println!("Selected cycle {aco:?}");
    println!("Total nodes: {}", aco.len());

    if args.print_aps {
        print_aps(&recs, &distances, &aco, dist, args.output);
    }

    if let Some(images_dir) = args.images {
        draw_images(images_dir, &airports, &apt_idx, &aco, args.unfiltered);
    }
}

const IMG_WIDTH: u32 = 1920 * 2;
const IMG_HEIGHT: u32 = 1080 * 2;

fn draw_images(
    mut images_dir: PathBuf,
    apts: &[Airport],
    apt_idx: &AirportIdx,
    aco: &[u32],
    draw_unfiltered: bool,
) {
    match images_dir.try_exists() {
        Ok(true) if images_dir.is_dir() => {}
        Ok(true) => {
            panic!("Images directory {images_dir:?} is not a directory");
        }
        Ok(false) => {
            panic!("Images directory {images_dir:?} does not exist");
        }
        Err(e) => {
            panic!("Images directory {images_dir:?} does not exist: {e:?}");
        }
    }

    let mut img_buf = RgbaImage::from_pixel(IMG_WIDTH, IMG_HEIGHT, Rgba([0xFF, 0xFF, 0xFF, 0xFF]));
    let (top_left, bottom_right) = apt_idx
        .aps
        .iter()
        .map(|apt| (apt.coord, apt.coord))
        .reduce(|(acc_tl, acc_br), (apt_tl, apt_br)| {
            (
                Coord {
                    lat: acc_tl.lat.max(apt_tl.lat),
                    lon: acc_tl.lon.min(apt_tl.lon),
                },
                Coord {
                    lat: acc_br.lat.min(apt_br.lat),
                    lon: acc_br.lon.max(apt_br.lon),
                },
            )
        })
        .unwrap();
    let margin = Coord {
        lon: (bottom_right.lon - top_left.lon).abs() * 0.05,
        lat: (bottom_right.lat - top_left.lat).abs() * 0.05,
    };
    let (top_left, bottom_right) = (
        Coord {
            lat: top_left.lat + margin.lat,
            lon: top_left.lon - margin.lon,
        },
        Coord {
            lat: bottom_right.lat - margin.lat,
            lon: bottom_right.lon + margin.lon,
        },
    );
    let scaler = Scaler::new(top_left, bottom_right, IMG_WIDTH, IMG_HEIGHT);
    images_dir.push("aco.png");

    for apt in if draw_unfiltered { apts } else { apt_idx.aps } {
        draw_hollow_circle_mut(
            &mut img_buf,
            scaler.map(apt.coord),
            5,
            Rgba([0xFF, 0, 0, 0xFF]),
        );
    }
    for (&aco1, &aco2) in cycling(aco) {
        draw_antialiased_line_segment_mut(
            &mut img_buf,
            scaler.map(apt_idx.aps[aco1 as usize].coord),
            scaler.map(apt_idx.aps[aco2 as usize].coord),
            Rgba([0, 0, 0xFF, 0xFF]),
            interpolate,
        );
    }
    let font = FontRef::try_from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/fonts/DejaVuSans.ttf"
    )))
    .unwrap();
    let font_height = 10.0;
    let scale = PxScale {
        x: font_height,
        y: font_height,
    };
    for apt in apt_idx.aps {
        let (x, y) = scaler.map(apt.coord);
        draw_text_mut(
            &mut img_buf,
            Rgba([0, 0, 0, 0xFF]),
            x + 5,
            y - 10 - 5,
            scale,
            &font,
            &apt.icao,
        );
    }
    let img_buf: RgbImage = img_buf.convert();
    img_buf.save(images_dir).unwrap();
}

fn print_aps<'a: 'b, 'b>(
    recs: &'b [AirportPrimaryRecord<'a>],
    distances_idx: &DistancesIdx,
    aco: &[u32],
    selected_dist: f64,
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
    writeln!(&mut writable, "Total lengths: {selected_dist:.05}").unwrap();
}

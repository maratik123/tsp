# TSP

This tool helps to parse Coded Instrument Flight Procedures (CIFP) and build optimal pathway
between airports using applied filters. With option to render them on image.

Usage:

1. Download and extract [FAACIFP18](https://www.faa.gov/air_traffic/flight_info/aeronav/digital_products/cifp/download/)
   or similar file from other administrations in ARINC 424 format.
2. Install [Rust](https://www.rust-lang.org/tools/install).
3. Open console from this directory.
4. Run this console tool to generate optimal pathway.

Help is available via `-h`/`--help` option:

```
cargo run --help

Usage: tsp [OPTIONS] [INPUT]

Arguments:
  [INPUT]  The input file. With no input file, or when input file is -, read standard input [default: -]

Options:
  -o, --output <OUTPUT>            Output file. If omitted, write to standard output
  -p, --print-aps                  Output airport primary records
  -f, --filter <FILTER>            Filter file
  -a, --ants <ANTS>                Number of ants [default: 50]
  -i, --iterations <ITERATIONS>    Number of iterations [default: 100]
  -e, --evaporation <EVAPORATION>  Evaporation rate (from 0 to 1) [default: 0.1]
      --alpha <ALPHA>              Alpha [default: 0.9]
      --beta <BETA>                Beta [default: 1.5]
  -u, --unfiltered                 Show unfiltered
      --images <IMAGES>            Output images directory
  -m, --min-dist <MIN_DIST>        Minimal allowable distance
  -h, --help                       Print help
  -V, --version                    Print version
```

Usage examples (considering, that file `FAACIFP18` lays in this directory):

1. Generate cyclic pathway between top largest and medium US hubs, with minimal leg distance 500 km. Generated
   image (`aco.png`) will be placed in current directory:

  ```bash
  cargo run --release -- FAACIFP18 -f res/large_medium_hubs -a 30 -i 100000 -e 0.1 -p --alpha 1 --beta 3 --images . -m 500
  ```

2. Generate almost optimal cyclic pathway between top largest US hubs. Generated image (`aco.png`) will be placed
   in `img` directory:

  ```bash
  cargo run --release -- FAACIFP18 -f res/large_hubs -a 30 -i 100000 -e 0.1 -p --alpha 1 --beta 3 --images ./img
  ```

3. Generate optimal as much as possible pathway between all airports from `FAACIFP18`:

  ```bash
  cargo run --release -- FAACIFP18 -a 30 -i 1000 -e 0.1 -p --alpha 1 --beta 3 --images ./img
  ```

# Examples

See [here](examples/README.md).

# Background

This utility parses data in ARINC 424 format.

Then builds airport records and filters them using provided list from file,
mentioned in `-f` key (just column of ICAO identifiers, see examples in [res](res) directory).

After that utility tries to build as much as possible optimal pathway between filtered records.
Although it is an NP-problem (you need to compare all combinations of possible cycles from data),
but there is exists methods that helps to find almost optimal solution. This utility uses [ant colony optimization
algorithm](https://en.wikipedia.org/wiki/Ant_colony_optimization_algorithms) to solve
[traveling salesman problem](https://en.wikipedia.org/wiki/Travelling_salesman_problem)

Parameters that affects the speed to find suitable result is:

* `-e` - evaporation factor, typical `0.1` is ok,
* `--alpha` - pheromone factor to select next possible leg, values between `0.9` and `1.1` is ok.
* `--beta` - distance factor to select next possible leg, values between `2` and `5` is ok,
* `--ants` - amount of ants in colony during one iteration, values between `30` and `50` is ok,
  but it depends on total count points (airports).
* `-i` - count of iterations to find solution. The more is better, try to use values between `1000` and `10000000`.
  YMMV on your CPU.

NB:

1. May be it is needed to run more than once, because the solutions on each iteration has the tendency to lock up on
   local optimum (not global).
2. It is useful to define environment variable `RUSTFLAGS="-C target-cpu=native"` during utility run.
   Also consider to use [Link time optimization](https://doc.rust-lang.org/cargo/reference/profiles.html#lto).

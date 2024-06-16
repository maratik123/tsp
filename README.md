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

```bash
cargo run --help
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

# Background

This utility parses data in ARINC 424 format.

Then build airport records and filters them using provided list from file,
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

NB: May be it is needed to run more than once, because the solutions on each iteration has the tendency to lock up on
local optimum (not global).

Gaia Sky LOD catalog generator
==============================

This project contains a re-implementation of the Gaia Sky LOD catalog generation in Rust. This version runs faster and consumes much less memory than its Java counterpart. The original Java implementation (now obsolete and outdated!) can be found in the main [Gaia Sky repository](https://codeberg.org/gaiasky/gaiasky/src/branch/master/core/src/gaiasky/data/octreegen). Preliminary test runs show a x2 increase in speed and a drastic reduction on the memory consumption (a factor of ~0.2) compared to the Java version.

Build
-----

Build the project with:

```bash
cargo build
```

If you need to build for release, do:

```bash
cargo build --release
```

Run 
---

You can run the catalog generator with directly with `cargo`:

```bash
cargo run
```

Usage
-----

Below are the CLI arguments:

```bash
Usage:
  target/debug/gaiasky-catgen [OPTIONS]

Generate LOD catalogs for Gaia Sky.

Optional arguments:
  -h,--help             Show this help message and exit
  -v,--version          Print version information
  -i,--input INPUT      Location of the input catalog
  -o,--output OUTPUT    Output folder. Defaults to system temp. If --dryrun is
                        present, this location is used to store the log
  --maxpart MAXPART     Maximum number of objects in an octant
  --plxerrfaint PLXERRFAINT
                        Parallax error factor for faint stars (gmag>=13.1),
                        where filter [plx_err/plx < plxerrfaint] is enforced
  --plxerrbright PLXERRBRIGHT
                        Parallax error factor for bright stars (gmag<13.1),
                        where filter [plx_err/plx < plxerrbright] is enforced
  --plxzeropoint PLXZEROPOINT
                        Parallax zero point
  -c,--skipmagcorrections
                        Skip magnitude and color corrections for extinction and
                        reddening
  --allownegativeplx    Allow negative parallaxes (and set them to 0.04 mas, or
                        25 Kpc) for Gaia stars
  -p,--postprocess      Post-process tree so that low-count nodes are merged
                        with their parents. See --childcount and --parentcount
                        for more info
  --childcount CHILDCOUNT
                        If --postprocess is on, children nodes with less than
                        --childcount objects and whose parent has less than
                        --parentcount objects will be merged with their parent.
                        Defaults to 100
  --parentcount PARENTCOUNT
                        If --postprocess is on, children nodes with less than
                        --childcount objects and whose parent has less than
                        --parentcount objects will be merged with their parent.
                        Defaults to 1000
  --hip HIP             Absolute or relative location of the Hipparcos catalog
                        (only csv supported)
  --distcap DISTCAP     Maximum distance in parsecs. Stars beyond this limit
                        are ignored
  --additional ADDITIONAL
                        Comma-separated list of files or folders with
                        optionally gzipped csv files containing additional
                        columns (matched by id) of the main catalog. The first
                        column must contain the Gaia source_id
  --xmatchfile XMATCHFILE
                        Crossmatch file between Gaia and Hipparcos, containing
                        two columns: source_id and hip
  --ruwe RUWE           RUWE threshold value. Filters out all stars with RUWE
                        greater than this value. If present, --plxerrfaint and
                        --plxerrbright are ignored.
  --columns COLUMNS     Comma-separated list of column names, in order, of the
                        Gaia catalog
  --filescap FILESCAP   Maximum number of input files to be processed
  --starscap STARSCAP   Maximum number of stars to be processed per file
  --dryrun              Dry run, do not write anything
  -d,--debug            Set log to debug
```

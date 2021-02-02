Gaia Sky LOD catalog generator
==============================

This project contains a re-implementation of the Gaia Sky LOD catalog generation in Rust. This is currently a WIP. A full (Java) implementation can be found in the main [Gaia Sky repository](https://gitlab.com/langurmonkey/gaiasky).

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
  -o,--output OUTPUT    Output folder. Defaults to system temp
  --maxpart MAXPART     Maximum number of objects in an octant
  --plxerrfaint PLXERRFAINT
                        Parallax error factor for faint stars (gmag>=13.1),
                        where filter [plx_err/plx < plxerrfaint] is enforced
  --plxerrbright PLXERRBRIGHT
                        Parallax error factor for bright stars (gmag<13.1),
                        where filter [plx_err/plx < plxerrbright] is enforced
  -c,--skipmagcorrections
                        Skip magnitude and color corrections for extinction and
                        reddening
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
                                                                                                                                                                                                                                                                                                                ```

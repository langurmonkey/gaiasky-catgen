# Catalog packer

This directory contains a small utility that helps compress and pack the generated catalogs into bundles that can be used by Gaia Sky by creating the right metadata files and moving the files around.

Here is how to use it:

```bash
Usage: catalog-pack.sh LOCATION KEY NAME DESCRIPTION RELEASENOTES EPOCH VERSION

    LOCATION      Location in the file system. Must contain log, metadata.dat, particles.
    KEY           The dataset key, which is also its file system name (dr2-small).
    NAME          The dataset name (DR2 small).
    DESCRIPTION   The description of the dataset.
    RELEASENOTES  Release notes.
    EPOCH        The reference epoch.
    VERSION      The version number.

Example:
catalog-pack.sh ./000-20220531-dr3-default dr3-default 'DR3 default' 'Gaia DR3 default: 20%\/1.5% bright\/faint parallax relative error.' '- Contains Hipparcos stars.\\n- When available, photometric distances are used.\\n- Parallaxes are using the corrected terms.' 2016.0 0
```

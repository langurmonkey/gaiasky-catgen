# Catalog packer

This directory contains a small utility that helps compress and pack the generated catalogs into bundles that can be used by Gaia Sky by creating the right metadata files and moving the files around.
It uses the metadata in the 'conf/catalogs-*.json' files.

Here is how to use it:

```bash
Usage: pack KEY DIR

	KEY     The catalog simple key, as a single word (i.e. 'default').
	DIR     Directory containing the generated catalog.

Example:
pack default ./000-20220531-dr3-default
```

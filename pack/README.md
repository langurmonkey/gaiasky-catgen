# Catalog packer

This directory contains a small utility that helps compress and pack the generated catalogs into bundles that can be used by Gaia Sky by creating the right metadata files and moving the files around.
It uses the metadata in the `conf/catalogs-*.json` files.

Here is how to use it:

```bash
Usage: pack KEY DIR [METADATA_FILE]

	KEY             The catalog simple key, as a single word (i.e. 'default').
	DIR             Directory containing the generated catalog.
	METADATA_FILE   Optional, path to the catalog metadata JSON file.
	                Defaults to conf/catalogs-dr4rc3.json.

Example:
pack default ./000-20220531-dr3-default
pack best ./000-20260918-dr4-best conf/catalogs-dr4rc3.json
```

## Metadata attributes

The `metadata` object of each catalog in `conf/catalogs-*.json` supports the
following attributes. All the old ones are still supported; the new ones are
marked.

| Attribute       | Required | Type            | Description |
|-----------------|----------|-----------------|-------------|
| `KEY`           | yes      | string          | Single-word lookup key used to find the entry in the metadata file (e.g. `best`). **Not** the dataset key. |
| `key` (metadata) | yes      | string          | Dataset key (e.g. `gaia-dr3-small`). This is the value used for the dataset key, folder names, and archive names — not the single-word lookup key passed to `pack`. |
| `name`          | yes      | string          | Dataset name. |
| `type`          | yes      | string          | Dataset type (`catalog-lod`, `catalog-gaia`, etc.). |
| `description`   | yes      | string          | Dataset description. |
| `epoch`         | yes      | number          | Reference epoch. |
| `version`       | yes      | number          | Version number. |
| `mingsversion`  | no       | number          | Minimum Gaia Sky version. Defaults to `3060100`. |
| `releasenotes`  | no       | string or array | Release notes. A string (old) or an array of strings (new). |
| `link`          | no       | string          | Single catalog link (**legacy**, still supported). |
| `links`         | no       | array of strings | Catalog links. Takes precedence over `link`. If only `link` is present, it is used as a single-element `links` array. |
| `replaces`      | no       | string          | Key of the dataset this one replaces (e.g. `gaia-dr3-best`). |
| `creator`       | no       | string          | Dataset creator. |
| `credits`       | no       | array of strings | Credits. |

Notes:

- If both `link` and `links` are present, `links` wins; the first element of
  `links` is also exposed as `link` in the generated `dataset.json` for
  compatibility with older Gaia Sky versions.
- Optional attributes that are absent are omitted from the generated
  `dataset.json` rather than emitted as empty strings.
- The generated `dataset.json` is validated as JSON before packing; the
  packing aborts if it is invalid.

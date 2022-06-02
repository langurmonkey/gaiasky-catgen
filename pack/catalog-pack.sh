#!/bin/bash

me=`basename "$0"`

function usage() {
echo "Usage: $me LOCATION KEY NAME DESCRIPTION RELEASENOTES EPOCH VERSION [LINK]"
echo
echo "	LOCATION      Location in the file system. Must contain log, metadata.dat, particles."
echo "	KEY           The dataset key, which is also its file system name (dr2-small)."
echo "	NAME          The dataset name (DR2 small)."
echo "	DESCRIPTION   The description of the dataset."
echo "	RELEASENOTES  The release notes."
echo "	EPOCH         The reference epoch."
echo "	VERSION       The version number."
echo "	LINK          Optional, the catalog link metadata."
echo
echo "Example:"
echo "$me gscatalogpack ./000-20220531-dr3-default dr3-default 'DR3 default' 'Gaia DR3
default: 20%\/1.5% bright\/faint parallax relative error.' '- Contains Hipparcos
stars.\\n- When available, photometric distances are used.\\n- Parallaxes are using the
corrected terms.' 2016.0 0 'https:\/\/gaia.ari.uni-hedielberg.de'"
}


SCRIPT_FILE=$(readlink -f "$0")
SCRIPT_DIR=$(dirname $SCRIPT_FILE)

if [ "$#" -ne 7 ] || [ "$#" -ne 8 ]; then
	usage
	exit 1
fi

LOCATION=$1
KEY=$2
NAME=$3
DESCRIPTION=$4
NOTES=$5
EPOCH=$6
VERSION=$7

# Link is optional
if [ "$#" -eq 7 ]; then
  # Default link to the repository.
  LINK="https://gaia.ari.uni-heidelberg.de/gaiasky/files/repository"
else
  LINK=$8
fi

if [ ! -d "$LOCATION" ] || [ ! -d "$LOCATION"/particles ]; then
	echo "ERROR: location does not exist or it does not contain a dataset: $LOCATION"
	exit 1
fi
case "$KEY" in
	*\ *)
		echo "ERROR: dataset key can not contain spaces: $KEY"
		exit 1
		;;
esac

CATALOG_FOLDER=$LOCATION/catalog
CATALOG_FILE=$LOCATION/catalog-$KEY.json
CATALOG=$CATALOG_FOLDER/$KEY

echo "CATALOG_FOLDER: $CATALOG_FOLDER"
echo "CATALOG_FILE: $CATALOG_FILE"

# PARSE DATA AND CHECK VALUES

# Get size in bytes of dataset
SIZE_BYTES=$(set -- $(du -b --max-depth=1 $LOCATION) && AUXVAR=$(( $# - 1 )) && echo ${!AUXVAR})
# Get particles
NOBJECTS=$(set -- $(grep Particles: $LOCATION/log) && echo ${!#})


# Check values
if [ -z "$VERSION" ]; then
    echo "ERROR: Version is empty"
    exit 1
fi
if [ -z "$SIZE_BYTES" ]; then
    echo "ERROR: Size (bytes) is empty"
    exit 1
fi
if [ -z "$NOBJECTS" ]; then
    echo "ERROR: Nobjects is empty"
    exit 1
fi

echo "SIZE:         $SIZE_BYTES bytes"
echo "NOBJECTS:     $NOBJECTS"
echo "EPOCH:        $EPOCH"
echo "VERSION:      $VERSION"

# CREATE AND MOVE CATALOG
mkdir -p $CATALOG
mv $LOCATION/log $LOCATION/metadata.bin $LOCATION/particles $CATALOG

# PREPARE JSON DESCRIPTOR FILE
cp $SCRIPT_DIR/catalog-template.json $CATALOG_FILE
sed -i 's/<NAME>/'"$NAME"'/g' $CATALOG_FILE
sed -i 's/<KEY>/'"$KEY"'/g' $CATALOG_FILE
sed -i 's/<VERSION>/'"$VERSION"'/g' $CATALOG_FILE
sed -i 's/<EPOCH>/'"$EPOCH"'/g' $CATALOG_FILE
sed -i 's/<DESCRIPTION>/'"$DESCRIPTION"'/g' $CATALOG_FILE
sed -i 's/<NOTES>/'"$NOTES"'/g' $CATALOG_FILE
sed -i 's/<LINK>/'"$LINK"'/g' $CATALOG_FILE
sed -i 's/<SIZE_BYTES>/'"$SIZE_BYTES"'/g' $CATALOG_FILE
sed -i 's/<NOBJECTS>/'"$NOBJECTS"'/g' $CATALOG_FILE

# TAR
TAR_FILE=catalog-$KEY.tar.gz
cd $LOCATION
tar -czvf $TAR_FILE catalog catalog-$KEY.json

set -- $(md5sum "$TAR_FILE") && echo $1 > md5
set -- $(sha256sum "$TAR_FILE") && echo $1 > sha256
cd -

echo "Done: $LOCATION/$TAR_FILE"

#!/usr/bin/env bash

# This script runs catalogs defined in catalogs-def.json. It needs to be called
# with some variables set: LOGS_LOC, DATA_LOC, DR_BASE, DR_LOC, COLS, CATDEF
# Dependencies: jq
# You must copy this script and the definition to your $GSC folder for it to work properly

# Get script path
SOURCE="${BASH_SOURCE[0]}"
while [ -h "$SOURCE" ]; do # resolve $SOURCE until the file is no longer a symlink
  GSDIR="$( cd -P "$( dirname "$SOURCE" )" && pwd )"
  SOURCE="$(readlink "$SOURCE")"
  [[ $SOURCE != /* ]] && SOURCE="$GSDIR/$SOURCE" # if $SOURCE was a relative symlink, we need to resolve it relative to the path where the symlink file was located
done

GSDIR="$( cd -P "$( dirname "$SOURCE" )/.." && pwd )"

usage() {
    echo "Usage: $0 [-c catalog_1,catalog_2,...] [-n max_files] [-h]"
    echo
    echo "    OPTIONS:"
    echo "       -c    comma-separated list of catalog names (default,small,medium,bright,large,verylarge,extralarge,ratherlarge,ruwe,full)"
    echo "       -n    maximum number of files to load, negative for unlimited"
    echo "       -h    show this help"
    1>&2; exit 1;
}

NFILES=-1

while getopts ":c:n:h" arg; do
    case $arg in
        c)
            CATALOGS=${OPTARG}
            ;;
        n)
            NFILES=${OPTARG}
            ;;
        h)
            usage
            ;;
        *)
            usage
            ;;
    esac
done
# Datasets to generate. Passed via arguments.
# Values: default, small, medium, bright, large, verylarge, extralarge, ruwe, full, fidelity
if [ -z "$CATALOGS" ]; then
    TORUN=("small" "default")
    echo "Using default catalog list: ${TORUN[*]}"
else
    IFS=',' read -r -a TORUN <<< "$CATALOGS"
    echo "Using user catalog list: ${TORUN[*]}"
fi

function generate() {
  echo "GENERATING: $DSNAME"
  echo "Input: $INPUT"
  echo "Output: $OUTPUT/$DSNAME/"
  echo "Log: $LOGS_LOC/$DSNAME.out"
  echo "Cmd: $CMD"

  $( eval $CMD )
}

NCAT=$(jq '. | length' $CATDEF)

for CURRENT_CATALOG in "${TORUN[@]}"; do
    for ((j=0;j<NCAT;j++)); do
        # Get catalog name
        NAME=$(jq ".[$j].name" $CATDEF)
        # Remove quotes
        NAME=$(sed -e 's/^"//' -e 's/"$//' <<<"$NAME")

        jpad=$(printf "%03d" $j)

        if [ "$NAME" == "$CURRENT_CATALOG" ]; then
            DSNAME="$jpad-$(date +'%Y%m%d')-$CATALOG_NAME-$NAME"
            echo $DSNAME
            CMD="nohup $GSDIR/target/release/gaiasky-catgen -i $INPUT -o $OUTPUT/$DSNAME/"
            NATTR=$(jq ".[$j] | length" $CATDEF)
            for ((k=0;k<NATTR;k++)); do
                KEY=$(jq ".[$j] | keys[$k]" $CATDEF)
                # Remove quotes
                KEY=$(sed -e 's/^"//' -e 's/"$//' <<<"$KEY")
                if [ "$KEY" != "name" ] && [ "$KEY" != "metadata" ]; then
                    VAL=$(jq ".[$j].$KEY" $CATDEF)
                    # Remove quotes
                    VAL=$(sed -e 's/^"//' -e 's/"$//' <<<"$VAL")
                    #echo "$KEY -> $VAL"
                    if [ "$VAL" == "null" ]; then
                        CMD="$CMD --$KEY"
                    else
                        CMD="$CMD --$KEY $VAL"
                    fi
                fi
            done
            CMD="$CMD --columns $COLS --filescap $NFILES > $LOGS_LOC/$DSNAME.out"
            generate
        fi
    done
done

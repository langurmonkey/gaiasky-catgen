#!/usr/bin/env python

import sys, os, argparse, struct

parser = argparse.ArgumentParser(description="Print information from generated particle binary files.",
                                 formatter_class=argparse.ArgumentDefaultsHelpFormatter)
parser.add_argument("src", help="Source directory where the 'particles_*.bin files are.")
args = parser.parse_args()
config = vars(args)

# assign directory
directory = args.src


print("Directory: %s" % directory)
print()

# iterate over files in
# that directory
for filename in os.listdir(directory):
    if filename.endswith(".bin"):
        f = os.path.join(directory, filename)
        with open(f, mode='rb') as file:
            fileContent = file.read()
            (marker, version, stars) = struct.unpack(">iii", fileContent[:12])
            print("%s >  mk: %i, v: %d, #stars: %s" % (filename, marker, version, stars))


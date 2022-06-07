#!/usr/bin/env python

import sys, os, argparse, struct

parser = argparse.ArgumentParser(description="Print information from generated particle binary files. If the input is a directory, it prints the version and number of stars of all files. If the input is a single file, it prints the stars in it.",
                                 formatter_class=argparse.ArgumentDefaultsHelpFormatter)
parser.add_argument("src", help="Source directory where the 'particles_*.bin files are.")
args = parser.parse_args()
config = vars(args)

input = args.src

if os.path.isdir(input):
    directory = input
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

elif os.path.isfile(input):
    file = input
    print("File: %s" % input)
    print()

    filename = os.path.basename(file)
    if filename.endswith(".bin") and filename.startswith("particles_"):
        with open(file, mode='rb') as f: 
            fileContent = f.read()
            (marker, version, stars) = struct.unpack(">iii", fileContent[:12])
            print("%s >  mk: %i, v: %d, #stars: %s" % (filename, marker, version, stars))

            pt = 12
            for st in range(stars):
                (x, y, z) = struct.unpack(">ddd", fileContent[pt:pt+24])
                pt += 24
                (pmx, pmy, pmz) = struct.unpack(">fff", fileContent[pt:pt+12])
                pt += 12
                (pmra, pmdec, rv) = struct.unpack(">fff", fileContent[pt:pt+12])
                pt += 12
                (appmag, absmag, col, size) = struct.unpack(">ffff", fileContent[pt:pt+16])
                pt += 16
                (hip, id) = struct.unpack(">iq", fileContent[pt:pt+12])
                pt += 12
                nlen = struct.unpack(">i", fileContent[pt:pt+4])
                pt += 4
                buf = []
                n = ""
                for ni in range(nlen[0]):
                    a = struct.unpack(">H", fileContent[pt:pt+2])
                    pt += 2
                    buf.append(a[0])

                names = "".join(map(chr, buf))

                print("id: %d - names: '%s' - mag: %f" % (id, names, appmag))
        
    else:
        print("File %s does not have the form 'particles_[id].bin" % filename)




#!/usr/bin/env python

import argparse, struct
from dataclasses import dataclass

parser = argparse.ArgumentParser(description="Print the octree structure from a metadata file.",
                                 formatter_class=argparse.ArgumentDefaultsHelpFormatter)
parser.add_argument("src", help="The metadata.bin file.")
args = parser.parse_args()
config = vars(args)

metadata = args.src

@dataclass
class Node:
    id: int
    x: float
    y: float
    z: float
    sx: float
    sy: float
    sz: float
    c1: int
    c2: int
    c3: int
    c4: int
    c5: int
    c6: int
    c7: int
    c8: int
    l: int
    nr: int
    n: int
    nc:int


node_size = 8 + 4*6 + 8*8 + 4*4

nodes = {}

def print_node(id: int, parent_idx: int):
    n = nodes[id]
    pad = "    " * n.l
    print("%s%d:L%d id:%d Obj(own/rec):(%d/%d) Nchld:%d" % (pad, parent_idx, n.l, n.id, n.n, n.nr, n.nc))

    if n.nc > 0:
        if n.c1 > 0:
            print_node(n.c1, 0)
        if n.c2 > 0:
            print_node(n.c2, 1)
        if n.c3 > 0:
            print_node(n.c3, 2)
        if n.c4 > 0:
            print_node(n.c4, 3)
        if n.c5 > 0:
            print_node(n.c5, 4)
        if n.c6 > 0:
            print_node(n.c6, 5)
        if n.c7 > 0:
            print_node(n.c7, 6)
        if n.c8 > 0:
            print_node(n.c8, 7)


if metadata.endswith("metadata.bin"):
    with open(metadata, mode='rb') as md:
        fileContent = md.read()
        (marker, version, nnodes) = struct.unpack(">iii", fileContent[:12])
        print("File: %s" % metadata)
        print("maker: %i, version: %d, number of nodes: %d" % (marker, version, nnodes))
        print()

        first = 0
        for i in range(nnodes):
            st = 12 + i * node_size
            (id,x,y,z,sx,sy,sx,c1,c2,c3,c4,c5,c6,c7,c8,l,nr,n,nc) = struct.unpack(">qffffffqqqqqqqqiiii", fileContent[st:st+node_size])
            nodes[id] = Node(id,x,y,z,sx,sy,sx,c1,c2,c3,c4,c5,c6,c7,c8,l,nr,n,nc)
            if i == 0:
                first = id 

        print_node(first, 0)

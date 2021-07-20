#!/bin/bash

example="roundabout_tiny"
features="tiny svg"

paths=$(cargo run --example "$example" --features "$features" -- -v -w 800 -h 600 --with-svg $@) || exit

for p in $paths ; do
    if [ ${p: -4} == ".svg" ] ; then
        mv $p ../../cause-effect.link/usezola/static/examples/roundabout/basic.svg
        break
    fi
done

for p in $paths ; do
    if [ ${p: -4} == ".png" ] ; then
        eog -f $p
        break
    fi
done

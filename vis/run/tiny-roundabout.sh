#!/bin/bash

example="roundabout_tiny"
features="tiny svg"
basic_svg="../../cause-effect.link/usezola/static/examples/roundabout/basic.svg"

paths=$(cargo run --example "$example" --features "$features" -- -v -w 800 -h 600 --with-svg $@) || exit

for p in $paths ; do
    if [ ${p: -4} == ".svg" ] ; then
        mv $p $basic_svg
        eog -f $basic_svg
        break
    fi
done

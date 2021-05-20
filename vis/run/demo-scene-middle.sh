#!/bin/bash

example="demo_scene_tiny"
features="tiny svg"

outpath=$(cargo run --example $example --features "$features" -- -v -w 800 -h 600 --with-svg --theme dark --amount 0.5 $@) || exit
eog -f "$outpath"

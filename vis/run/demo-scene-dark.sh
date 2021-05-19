#!/bin/bash

example="demo_scene_usvg"

outpath=$(cargo run --example $example -- -v -w 800 -h 600 --with-svg --theme dark $@) || exit
eog -f "$outpath"

#!/bin/bash

example="demo_scene"

out_path=$(cargo run --example $example -- -v -w 800 -h 600 --with-svg --theme dark $@) && eog -f ${out_path}

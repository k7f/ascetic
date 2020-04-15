#!/bin/bash

example="demo_scene"

out_path=$(cargo run --example $example -- -v -w 800 -h 600 --with-svg --theme dark --amount 0.5 $@) && eog -f ${out_path}

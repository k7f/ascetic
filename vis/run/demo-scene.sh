#!/bin/bash

outpath=$(cargo run --example demo_scene -- -v -w 800 -h 600 --with-svg $@) || exit
eog -f "$outpath"

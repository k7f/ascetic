#!/bin/bash

out_path=$(cargo run --example demo_scene -- -v -w 800 -h 600 --with-svg $@) && eog -f ${out_path}

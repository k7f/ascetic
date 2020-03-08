
#!/bin/bash

example="simple"

out_path=$(cargo run --example $example -- -v -w 800 -h 600 --with-svg) && eog -f ${out_path}

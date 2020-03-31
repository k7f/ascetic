#!/bin/bash

example="dump"
pnml="../data/pnml/test.pnml"

cargo run --example $example -- $pnml $@

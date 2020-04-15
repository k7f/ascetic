#!/bin/bash

if [ -z ${SCRIPT_PATH} ] ; then
    if [ -d "scripts" ] ; then
        SCRIPT_PATH="scripts"
    else
        SCRIPT_PATH="."
    fi
fi

cargo run --bin ascesis -- $@

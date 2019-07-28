#!/bin/bash

for makefile in $(find . -name "Makefile" -exec dirname {} +); do
    pushd "$makefile" && make && popd
done

for library in $(find ./chess -name "*.so"); do
    name=`basename $library`
    cp "$library" "../venv/src/chess/$name"
done


for library in $(find ./models -name "*.so"); do
    name=`basename $library`
    cp "$library" "../venv/src/models/$name"
done

#!/bin/bash

for makefile in $(find ./c_src -name "Makefile" -exec dirname {} +); do
    pushd "$makefile" && make && popd
done

for library in $(find ./c_src/chess -name "*.so"); do
    name=`basename $library`
    cp "$library" "./py_src/chess/$name"
done


for library in $(find ./c_src/models -name "*.so"); do
    name=`basename $library`
    cp "$library" "./py_src/models/$name"
done


for library in $(find ./c_src/harness -name "*.so"); do
    name=`basename $library`
    cp "$library" "./py_src/harness/$name"
done

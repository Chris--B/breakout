#!/bin/bash

SCRIPT_DIR=$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)
dir=${1:-.}

set -ex

# sdk can be: macosx, iphoneos
xcrun -sdk macosx metal         \
    -frecord-sources=flat       \
    $SCRIPT_DIR/Shaders.metal   \
    -o $dir/Shaders.metallib

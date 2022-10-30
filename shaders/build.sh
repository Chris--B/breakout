#!/bin/bash

SCRIPT_DIR=$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)
dir=${1:-.}

set -ex

export MTL_ENABLE_DEBUG_INFO=INCLUDE_SOURCE

# sdk can be: macosx, iphoneos
xcrun -sdk macosx metal         \
    -frecord-sources=flat       \
    -gline-tables-only -MO      \
    $SCRIPT_DIR/Shaders.metal   \
    -o $dir/Shaders.metallib

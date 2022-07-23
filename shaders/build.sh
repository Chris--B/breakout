#!/bin/bash

SCRIPT_DIR=$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)

set -ex

xcrun -sdk macosx metal         \
    -frecord-sources=flat       \
    $SCRIPT_DIR/Shaders.metal   \
    -o Shaders.metallib

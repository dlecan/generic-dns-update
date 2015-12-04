#!/bin/bash

set -ev

source ./scripts/config.sh

OS="linux"
ARCH="amd64"
BUILD_BIN_FILE="$(pwd)/target/release/gdu"
PKG_INSTALL_BIN_DIR="/usr/bin/"
PKG_NAME="$APP"
OUTPUT_DIR="$(pwd)/pkg/"

mkdir -p $OUTPUT_DIR

cargo build --release

fpm \
  -s dir \
  -t deb \
  -n $PKG_NAME \
  -p $OUTPUT_DIR \
  -v $VERSION \
  -a $ARCH \
  --vendor $VENDOR \
  $BUILD_BIN_FILE=$PKG_INSTALL_BIN_DIR

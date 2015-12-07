#!/bin/bash

set -ev

source ./scripts/config.sh

## AMD64 binary

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

## ARMv6 binary for Raspbian

OS="linux"
ARCH="armhf"
BUILD_BIN_FILE="$(pwd)/target/arm-unknown-linux-gnueabihf/release/gdu"

docker run -it --rm \
  -v $(pwd):/source \
  dlecan/rust-x86_64-armv6 \
  cargo build --release --target arm-unknown-linux-gnueabihf

fpm \
  -s dir \
  -t deb \
  -n $PKG_NAME \
  -p $OUTPUT_DIR \
  -v $VERSION \
  -a $ARCH \
  --vendor $VENDOR \
  $BUILD_BIN_FILE=$PKG_INSTALL_BIN_DIR

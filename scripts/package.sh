#!/bin/bash

set -ev

source ./scripts/config.sh

## AMD64 binary

OS="linux"
PKG_INSTALL_BIN_DIR="/usr/bin/"
PKG_NAME="$APP"
OUTPUT_DIR="$(pwd)/pkg/"

if [ ! -d "$OUTPUT_DIR" ]
then
  mkdir -p $OUTPUT_DIR

  cargo build --release

  ## 64 bits deb package
  ARCH="amd64"
  BUILD_BIN_FILE="$(pwd)/target/release/gdu"

  fpm \
    -s dir \
    -t deb \
    -n $PKG_NAME \
    -p $OUTPUT_DIR \
    -v $VERSION \
    -a $ARCH \
    --vendor $VENDOR \
    $BUILD_BIN_FILE=$PKG_INSTALL_BIN_DIR

  ## 64 bits rpm package
  ARCH="x86_64"
  BUILD_BIN_FILE="$(pwd)/target/release/gdu"

  fpm \
    -s dir \
    -t rpm \
    -n $PKG_NAME \
    -p $OUTPUT_DIR \
    -v $VERSION \
    -a $ARCH \
    --vendor $VENDOR \
    $BUILD_BIN_FILE=$PKG_INSTALL_BIN_DIR

  ## ARMv6 binary for Raspbian

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
fi


#!/bin/bash

set -ev

source ./scripts/config.sh

echo "Building for rust $TRAVIS_RUST_VERSION on $ARCH..."

if [ "$ARCH" = "x86_64" ]
then

  cargo build --verbose
  cargo test --verbose

elif [ "$ARCH" = "armv6" ]
then

  docker run -it --rm \
    -v $(pwd):/source \
    dlecan/rust-x86_64-armv6 \
    cargo build --verbose --target arm-unknown-linux-gnueabihf

  docker run -it --rm \
    -v $(pwd):/source \
    dlecan/rust-x86_64-armv6 \
    cargo test --verbose --target arm-unknown-linux-gnueabihf

else
  echo "Unknown architecture!"
fi

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
    -v ~/.cargo/git:/root/.cargo/git \
    -v ~/.cargo/registry:/root/.cargo/registry \
    dlecan/rust-crosscompiler-arm:stable \
    cargo build --verbose

#  docker run -it --rm \
#    -v $(pwd):/source \
#    -v ~/.cargo/git:/root/.cargo/git \
#    -v ~/.cargo/registry:/root/.cargo/registry \
#    dlecan/rust-crosscompiler-armv6:stable \
#    cargo test --verbose

else
  echo "Unknown architecture!"
fi

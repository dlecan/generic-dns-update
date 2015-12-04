#!/bin/bash

export APP='gandi-dns-updater'
export VENDOR='dlecan.com'

CARGO_VERSION=`grep -m 1 "version = \"[0-9.]*\"" Cargo.toml | sed -n 's/version = "\([0-9.]*\)"/\1/p'`

export VERSION="$CARGO_VERSION-$TRAVIS_BUILD_NUMBER-${TRAVIS_COMMIT:0:6}"

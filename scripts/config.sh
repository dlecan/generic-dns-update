#!/bin/bash

export APP='generic-dns-update'
export VENDOR='dlecan.com'

export CARGO_VERSION=`grep -m 1 "version = \"[0-9.]*\"" Cargo.toml | sed -n 's/version = "\([0-9.]*\)"/\1/p'`
export GIT_VERSION=${TRAVIS_COMMIT:0:6}

#!/bin/bash

set -e

CURRENT_DIR=$(echo $(cd $(dirname $0) && pwd))
PROJECT_ROOT="${CURRENT_DIR}/.."

if !(type "cross" > /dev/null 2>&1); then
  cargo install cross
fi


case "$(uname -s)" in
  "Darwin" )
    rustup target add x86_64-unknown-linux-musl
    ;;
  # crossがdarwinに対応したら書く
  # "Linux" )
  #   rustup target add x86_64-apple-darwin
  #   ;;
esac

cross build --release --target x86_64-unknown-linux-musl

# mac用のbuild
# crossは未対応 
# cross build --release --target x86_64-apple-darwin

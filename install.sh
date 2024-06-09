#!/bin/bash

# This script provides automated installation or updating of goup on your system. Simply clone
# this repo to your local system (like a temp directory) and execute this script from the root.
#
# ```
# git clone https://github.com/skubalj/goup.git
# cd goup
# ./install.sh
# ````
# 
# This script will install goup to $GOPATH/bin, while goup itself will add installed versions to
# $GOPATH/goup. As this script can be used to bootstrap a system with no existing go installation,
# it will create an environment variable file that can be called from your ~/.bashrc to configure
# your shell. You can customize the location of GOPATH by specifying the variable when running 
# this script.

echo "Checking for environment variables"
if [ -z "$GOPATH" ]; then
    GOPATH="$HOME/.go"
    echo "GOPATH not found. Defaulting to '$GOPATH'"
else
    echo "GOPATH found. Installing to '$GOPATH/goup'"
fi
GOUP_DIR="$GOPATH/goup"
GOROOT="$GOUP_DIR/go"

mkdir -p $GOUP_DIR
printf "export GOPATH=\"$GOPATH\" # The global dir for packages and installed binaries\n\
export GOROOT=\"$GOROOT\" # The installed Go development kit\n\
export PATH=\"${GOROOT}/bin:${GOPATH}/bin:\${PATH}\"\n\
" > $GOUP_DIR/env

echo "Compiling goup..."
cargo build --release -q
mkdir -p $GOPATH/bin
cp ./target/release/goup "$GOPATH/bin/goup"

echo "Installed successfully"
echo "If this is a first-time install, add '. "$GOPATH/goup/env"' to your ~/.bashrc"

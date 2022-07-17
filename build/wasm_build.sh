#!/bin/bash
set -e

# Install wasm-bindgen with `cargo install wasm-bindgen-cli`.
# Files in OutDir is everything needed to run the web page.

OutDir=target/wasm_package


if [ ! -e .git ]; then
	echo "Must be run from repository root"
	exit 1
fi

#
# Extract project name from Cargo.toml
#

ProjName="$(cargo metadata --no-deps --format-version 1 |
        sed -n 's/.*"name":"\([^"]*\)".*/\1/p')"

#
# Build
#

if [ ! -e target ] ; then
    mkdir target
fi

cargo build --release --target wasm32-unknown-unknown 

FileName="$ProjName.wasm"
WasmFile="$(cargo metadata --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')/wasm32-unknown-unknown/release/$FileName"

if [ ! -e "$WasmFile" ]; then
	echo "Script is borken, it expects file to exist: $WasmFile"
	exit 1
fi

[ ! -e "$OutDir" ] || rm -r "$OutDir"

BINDGEN_EXEC_PATH="${CARGO_HOME:-~/.cargo}/bin/wasm-bindgen"

if [ ! -e "$BINDGEN_EXEC_PATH" ] ; then
    echo "Please install wasm-bindgen, cannot generate the wasm output without it"
    exit 1
fi

$BINDGEN_EXEC_PATH \
	--no-typescript \
	--out-dir "$OutDir" \
	--target web \
	"$WasmFile"


if [ -e $(which wasm-opt) ] ; then
	BindgenOutput="$OutDir/${ProjName}_bg.wasm"
	wasm-opt -Oz -o "$BindgenOutput.post-processed" "$BindgenOutput"
	echo "Applying wasm-opt optimizations"
	echo "before: $(wc -c "$BindgenOutput")"
	echo "after : $(wc -c "$BindgenOutput.post-processed")"
	mv "$BindgenOutput.post-processed" "$BindgenOutput"
else
	echo "Continuing without wasm-opt, it is highly recommended that you"
	echo "install it, it has been known to divide by two wasm files size"
fi

cp build/wasm_build.html "$OutDir/index.html"

# Rename JS files
Count=0
for _ in $OutDir/*.js; do
	((Count+=1))
done

if [ $Count -ne 1 ]; then
	echo "Script is broken, must be 1 JS file matching mask"
	exit 1
fi

mv $OutDir/*.js "$OutDir/main.js"

if [ "$1" ] ; then
	trash-put "$1"
	mv "$OutDir" "$1"
fi

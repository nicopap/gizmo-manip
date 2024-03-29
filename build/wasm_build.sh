#!/bin/bash
set -e

# Install wasm-bindgen with `cargo install wasm-bindgen-cli`.
# Files in OutDir is everything needed to run the web page.

OutDir=target/wasm_package

ReleaseFlags="--release"
BinDirectory="release"
RunWasmopt=1
Target="$OutDir"

while true ; do
  case "$1" in
    --debug)
      ReleaseFlags=""
      BinDirectory="debug"
      RunWasmopt=0
      shift
    ;;
    --target)
      OutDir="$2"
      shift 2
    ;;
    --*)
      echo "ERROR: unknown option $1"
      exit 1
    ;;
    *)
      break
    ;;
  esac
done

if [ ! -e .git ]; then
  echo "Must be run from repository root"
  exit 1
fi

# Extract project name from Cargo.toml
ProjName="$(cargo metadata --no-deps --format-version 1 |
        sed -n 's/.*"name":"\([^"]*\)".*/\1/p')"

#
# Build
#

RUSTFLAGS='--cfg=web_sys_unstable_apis' cargo build $ReleaseFlags --target wasm32-unknown-unknown 

FileName="$ProjName.wasm"
WasmFile="$(cargo metadata --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')/wasm32-unknown-unknown/$BinDirectory/$FileName"

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


if [[ -e $(which wasm-opt) && $RunWasmopt == 1 ]] ; then
  BindgenOutput="$OutDir/${ProjName}_bg.wasm"
  echo "Applying wasm-opt optimizations"
  echo "before: $(wc -c "$BindgenOutput")"
  wasm-opt -Oz \
        --output "$BindgenOutput.post-processed" "$BindgenOutput"
  echo "after : $(wc -c "$BindgenOutput.post-processed")"
  mv "$BindgenOutput.post-processed" "$BindgenOutput"
elif [[ ! $RunWasmopt ]] ; then
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
  echo "Script is broken, wasm-bindgen didn't generate a *.js file"
  exit 1
fi

mv $OutDir/*.js "$OutDir/bevy.js"

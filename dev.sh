#!/bin/bash

set -e

# Kill all processes on exit and error.
trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

# Serve the HTML and JS files.
python -m SimpleHTTPServer 8080 2> /dev/null &

echo "Serving WebAssembly app on http://localhost:8080/"

# Recompile on changes.
watchexec --exts rs,html,toml 'wasm-pack build -t web'
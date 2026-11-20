#!/usr/bin/env bash
set -euo pipefail
cd /home/data/projects/pianoteq-eng-reverse2/piano-tect-tauri
exec ./scripts/with-sysroot.sh npm run tauri dev

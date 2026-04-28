#!/bin/bash
echo "🏃 Starting cargo watch for taskify_runner..."
cargo watch -w taskify_runner/src -x "run -p taskify_runner" & 
RUNNER_PID=$!
echo "🔨 Starting cargo watch for taskify_web..."
cargo watch -w taskify_web/src -x "run -p taskify_web" & 
WEBUI_PID=$!

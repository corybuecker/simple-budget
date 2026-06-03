#!/bin/bash

set -e

# Bacon is for automatically restarting the server when code changes are detected.
cargo install --locked bacon@3.23.0

# OSC Scanner is for checking for vulnerabilities in dependencies.
curl -L -o /workspaces/simple-budget/osv-scanner --progress-bar https://github.com/google/osv-scanner/releases/download/v2.3.8/osv-scanner_linux_arm64
echo "8158b18edd2d03b1a30d905ca91b032bc62262167be8f206c27114f08823e27c  /workspaces/simple-budget/osv-scanner" | shasum -a 256 --check || rm /workspaces/simple-budget/osv-scanner
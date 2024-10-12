#!/bin/bash

set -euo pipefail
IFS=$'\n\t'

rm -rf SimpleBudget.xcodeproj
xcodegen
open SimpleBudget.xcodeproj
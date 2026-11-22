#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo test --manifest-path handline-core/Cargo.toml
swift run --package-path ios/HandlineKit HandlineCheck
if [ -z "${JAVA_HOME:-}" ] && [ -d /opt/homebrew/opt/openjdk@17 ]; then
  export JAVA_HOME=/opt/homebrew/opt/openjdk@17
fi
(cd android && ./gradlew test --no-daemon)

#!/bin/sh
#
# Use this script to run your program LOCALLY.
#
# Note: Changing this script WILL NOT affect how CodeCrafters runs your program.
#
# Learn more: https://codecrafters.io/program-interface

set -e # Exit early if any commands fail

# Copied from .codecrafters/compile.sh
#
# - Edit this to change how your program compiles locally
# - Edit .codecrafters/compile.sh to change how your program compiles remotely
# (
#   cd "$(dirname "$0")" # Ensure compile steps are run within the repository directory
#   cargo build --release --target-dir=/tmp/codecrafters-build-redis-rust --manifest-path Cargo.toml
# )

# Copied from .codecrafters/run.sh
#
# - Edit this to change how your program runs locally
# - Edit .codecrafters/run.sh to change how your program runs remotely
# exec /tmp/codecrafters-build-redis-rust/release/codecrafters-redis "$@"
# printf "*1\r\n$4\r\nPING\r\n" | nc 127.0.0.1 6379
# printf '*3\r\n$3\r\nSET\r\n$5\r\nmykey\r\n$5\r\nhello\r\n' | nc 127.0.0.1 6379
# printf '*5\r\n$3\r\nSET\r\n$5\r\nmykey\r\n$5\r\nhello\r\n$2\r\nPX\r\n$4\r\n5000\r\n' | nc 127.0.0.1 6379
# printf '*5\r\n$3\r\nSET\r\n$6\r\norange\r\n$5\r\ngrape\r\n$2\r\nPX\r\n$3\r\n100\r\n' | nc 127.0.0.1 6379
# printf '*2\r\n$4\r\nECHO\r\n$4\r\npear\r\n' | nc 127.0.0.1 6379
# printf '*4\r\n$5\r\nRPUSH\r\n$9\r\nraspberry\r\n$6\r\nbanana\r\n$4\r\npear\r\n' | nc 127.0.0.1 6379
# printf '*3\r\n$5\r\nBLPOP\r\n$6\r\norange\r\n$1\r\n0\r\n' | nc 
# printf '*3\r\n$5\r\nBLPOP\r\n$9\r\npineapple\r\n$3\r\n0.1\r\n' | nc 127.0.0.1 6379
printf '*5\r\n$4\r\nXADD\r\n$9\r\nblueberry\r\n$3\r\n0-1\r\n$3\r\nfoo\r\n$3\r\nbar\r\n' | nc 127.0.0.1 6379
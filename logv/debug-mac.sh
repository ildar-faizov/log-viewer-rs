set -xe

cargo build && /Applications/Xcode.app/Contents/SharedFrameworks/LLDB.framework/Resources/debugserver 0.0.0.0:8888 ./target/debug/logv "$@"

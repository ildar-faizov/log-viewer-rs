## Debug

### Linux

Run program using
```
gdbserver :8888 ./target/debug/logv -f ./samples/syslog
```
or 
```
./debug.sh -f ./samples/syslog
```

Then attach remote GDB with `target remote` args `127.0.0.1:8888`

### MacOS

Install XCode.

Run program using
```
/Applications/Xcode.app/Contents/SharedFrameworks/LLDB.framework/Resources/debugserver 0.0.0.0:8888 ./target/debug/logv -f ./samples/syslog
```
or
```
./debug-mac.sh -f ./samples/syslog
```

Then attach remote LLDB with `target remote` args `connect://127.0.0.1:8888`


## Profiling

Profiling is implemented using [profiling](https://crates.io/crates/profiling) and 
[puffin](https://crates.io/crates/puffin). To use profiler:
- `cargo install puffin-viewer`
- run logv with `-p <PORT>`
- run puffin viewer with `--url http://localhost:PORT`

By default, it's convenient to use 8585, as puffin viewer uses this port by default.

## Developer's Notes

Print offsets of new lines in a file:
```shell
grep -obazP '\n' test.txt
```
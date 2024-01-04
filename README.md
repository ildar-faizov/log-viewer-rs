## Debug

Run program using
```
gdbserver :8888 ./target/debug/logv -f ./samples/syslog
```
or 
```
./debug.sh -f ./samples/syslog
```

Then attach remote GNU debugger with `target remote` args `127.0.0.1:8888`

## Profiling

Profiling is implemented using [profiling](https://crates.io/crates/profiling) and 
[puffin](https://crates.io/crates/puffin). To use profiler:
- `cargo install puffin-viewer`
- run logv with `-p <PORT>`
- run puffin viewer with `--url http://localhost:PORT`

By default it's convenient to use 8585, as puffin viewer uses this port by default.
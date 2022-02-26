## Debug

Run program using
```gdbserver :8888 ./target/debug/logv -f ./syslog```

Then attach remote GNU debugger with `target remote` args `127.0.0.1:8888`
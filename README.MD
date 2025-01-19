# Log Viewer (logv)

`logv` is a plain text file viewer, intended to show potentially large UTF-8 files (typically logs) and provide means 
to analyze them.

## Basic Usage

```shell
logv 
```
or
```shell
logv -f ./samples
```

## Features

- Quit (`q`)\
  [![asciicast](https://asciinema.org/a/fUYdgyRjt3rJL3Q3kycb4IMV6.svg)](https://asciinema.org/a/fUYdgyRjt3rJL3Q3kycb4IMV6)
- Open file (`Control+O`)\
  [![asciicast](https://asciinema.org/a/R93IcKx56ksXYXytncunkN6oG.svg)](https://asciinema.org/a/R93IcKx56ksXYXytncunkN6oG)
- Date recognition highlighting\
  [![asciicast](https://asciinema.org/a/04kNpZWeAK1dKjzkRrv7kxYUR.svg)](https://asciinema.org/a/04kNpZWeAK1dKjzkRrv7kxYUR)
- Search (`Control+F`)\
  [![asciicast](https://asciinema.org/a/gsB4iJ9iUcAwqyvTbUPnDEzHt.svg)](https://asciinema.org/a/gsB4iJ9iUcAwqyvTbUPnDEzHt)
- Filter lines by pattern (`Control+Y`)\
  [![asciicast](https://asciinema.org/a/MXZKWIh1NiSKym6nSgePJaf1W.svg)](https://asciinema.org/a/MXZKWIh1NiSKym6nSgePJaf1W)

## Build & Run

```shell
cargo run
```
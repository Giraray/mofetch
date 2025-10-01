# Changelog

## 1.2.0
### New
- Added a new user configuration `-I` `--hide-info`, which omits the system information from the process and only renders the ASCII thumbnail.
- Added a temporary fetch for updating CPU usage.

### Fixes
- Fixed a bug that improperly scales media when its dimensions are larger than the allowed values.

## 1.1.0
### New
- Implemented shader configurations with the arguments --brightness, --contrast, --no-edges and --edge-threshold. --edge-threshold can be used to determine the edge strength required for the algorithm to render a tile as an edge.

### Fixes
- Fixed an issue with ffprobe command not giving an output in get_frames()

## 1.0.0
- Project release
# Changelog

## 1.4.0
### New
- `mofetch/config.toml` is generated in your user config directory the first time you use mofetch. You can tweak the default settings for all user arguments here. You can also customize the system info key names and their values.

### Changes
- Changed some text in `--help` for clarity.

### Fixes
- Fixed a bug where the program would exit upon rendering static images, preventing the fetching and displaying of system info.

## 1.3.0
### New
- You can now use `--verbose` to see the flood of information generated during image processing.
- Added `--version` to quickly get mofetch version.

### Changes
- Framerate is now capped to source framerate as upscaling the framerate usually doesn't yield great results.
- System uptime is now updated periodically.

## 1.2.0
### New
- Added a new user configuration `-I` `--hide-info`, which omits the system information from the process and only renders the ASCII thumbnail.
- Added a temporary fetch for updating CPU usage.

### Fixes
- Fixed a bug that improperly scales media when its dimensions are larger than the allowed values.

## 1.1.0
### New
- Implemented shader configurations with the arguments `--brightness`, `--contrast`, `--no-edges` and `--edge-threshold`. `--edge-threshold` can be used to determine the edge strength required for the algorithm to render a tile as an edge.

### Fixes
- Fixed an issue with ffprobe command not giving an output in get_frames()

## 1.0.0
- Project release
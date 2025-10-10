mofetch is a Neofetch-like tool to fetch system information to a terminal, with the addition of an animated, user-generated ASCII thumbnail. The thumbnail is a [wgpu](https://github.com/gfx-rs/wgpu) implementation of [Acerola's ASCII algorithm](https://www.youtube.com/watch?v=gg40RWiaHRY),
utilizing the GPU to make beautiful edge-lines for the ASCII image.

### Dependencies
mofetch uses FFmpeg to process media in order to support a broad range of file formats. Ensure that it is installed on your machine before using mofetch.

### ASCII cache
Processing large media files, such as videos, into ASCII art can take a while. mofetch caches all processed thumbnails to a directory in the user cache folder (e.g. `$HOME/.cache` on linux), discarding the need to process files again the next time you'd like to use the same file.

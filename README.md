# LZW
A simple LZW Compressor using fixed-width 16 bit codes.
The program does not use a buffer, it just reads in the entire input file at once, so compressing large files is not recommended.
Uses '#' as EOF character.

To Compress:
cargo run c input output
To Decompress:
cargo run d input output

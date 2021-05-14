# LZW
A simple LZW Compressor using fixed-width 16 bit codes.<br>
The program does not use a buffer, it just reads in the entire input file at once, so compressing large files is not recommended.<br>
Uses '#' as EOF character.<br>

To Compress:<br>
cargo run c input output<br>
To Decompress:<br>
cargo run d input output<br>

# LZWv1
A simple LZW Compressor using fixed-width 16 bit codes.<br>
The program does not use a buffer, it just reads in the entire input file at once, so compressing large files is not recommended.<br>
Uses '#' as EOF character.<br>

To Compress:<br>
lzw.exe c input.txt output.txt<br>
To Decompress:<br>
lzw.exe d input.txt output.txt<br>

# LZWv2
Same as LZWv1, but with buffered reading and writing, and the decompressor reads 2 byte values instead of characters, so invalid code points are no longer an issue and the max dictionary size can be increased to 65535.<br>
Uses '#' as EOF character.<br>
<br>
To Compress:<br>
lzw2.exe c input.txt output.bin<br>
To Decompress:<br>
lzw2.exe d input.bin output.txt<br>

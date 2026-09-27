# Cache fixtures

`pixel.png` is a complete 2x2 RGB PNG generated with Pillow 11.3.0. All four pixels
are (32, 128, 224). It was reopened and fully decoded with Pillow after generation.
It contains 75 bytes, including IDAT and IEND, rather than just a PNG signature.

The two `.bin` files are synthetic Chromium Simple Cache version 5 combined
stream 0/1 entries following Chromium's authoritative layout:
https://chromium.googlesource.com/chromium/src/+/main/net/disk_cache/simple/simple_entry_format.h

Each contains a 24-byte little-endian header, a 59-byte partitioned cache key,
the complete PNG as stream 1, its 24-byte EOF, 41 bytes of stream-0 metadata,
and the final 24-byte EOF. `simple-v5-checks.bin` additionally contains the
32-byte SHA256 key digest and both stream CRC32 flags/checksums.
`simple-v5-unchecked.bin` omits these optional checks.

The key is `1/0/_dk_https://example.test https://example.test/pixel.png`.
Its header PersistentHash is 0xbc2c5e71, calculated with Chromium's SuperFastHash
algorithm. CRC32 and SHA256 values were independently generated using Python
`zlib` and `hashlib`, rather than the Rust parser implementation.

Stream 0 deliberately uses recognizable synthetic HTTP metadata ending in NUL
bytes. It is opaque to the extractor; these fixtures do not purport to be actual
Chrome HttpResponseInfo pickles. There is no personal browser data in the files.

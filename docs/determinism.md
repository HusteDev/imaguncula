# Determinism contract

Exporting the same project twice must produce byte-identical packages, as long as the source files, pinned preset versions, included outputs, accepted warnings, and Imaguncula version are the same.

This covers:

- PNG files and sanitized SVG files;
- the order, encoding, dimensions, and bytes of ICO and ICNS entries;
- generated JSON, XML, HTML, and text files, and `report.md`;
- the entry order, timestamps, attributes, and contents of the combined ZIP;
- every file path and byte in the exported package directory.

A core test exports the same project twice, compares the two ZIP files, and compares the two package directories file by file and byte by byte. Other tests repeat render, resize, and PNG encoding operations and reopen ICO and ICNS containers to check their contents.

## Limits of the contract

The contract applies to one Imaguncula version and identical inputs. Output may change between exports when:

- Imaguncula or one of its dependencies is upgraded;
- you accept a preset migration;
- a source file is renamed, because `report.md` records each source's file name.

File-system metadata outside the file contents, such as ownership and modification times, is not covered.

The contract is about exported icon packages. Imaguncula's own build is covered separately: building the application from the same source produces byte-identical executables, which CI checks on every platform (see [distribution.md](distribution.md#repeatable-builds)). The installers and archives that wrap those executables (MSI, NSIS, AppImage, DMG, and the CLI archives) still contain packaging timestamps; each release publishes their SHA-256 sums.

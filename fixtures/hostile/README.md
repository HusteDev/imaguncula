# Hostile input corpus

These files are intentionally invalid or unsafe. Never open them in a browser or image editor as trusted artwork.

| Fixture | Expected boundary |
| --- | --- |
| `script.svg` | rejects executable script elements |
| `external-entity.svg` | rejects document types and entity expansion |
| `foreign-object.svg` | rejects embedded active HTML surfaces |
| `event-handler.svg` | rejects event-handler attributes |
| `css-url.svg` | rejects or warns on resource-bearing CSS without fetching it |
| `malformed.svg` | rejects malformed XML |
| `traversal-paths.txt` | every line is rejected by the package-path sanitizer |

Oversized sparse files, excessive element counts, excessive nesting, PNG header bombs, truncated PNGs, symlinks, and staging-tree mutations are generated in tests so the repository does not carry large or platform-specific binary fixtures. New security regressions should add the smallest stable reproducer here or to the focused test that constructs it.

# Security Policy

## Supported versions

Security fixes are applied to the current v1 release line and `main`. Older versions receive fixes only when explicitly listed.

| Version | Supported |
| --- | --- |
| 1.x | Yes |
| Current `main` | Yes |
| 0.x, older commits, and unlisted builds | No |

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability.

Use **Report a vulnerability** in the repository's Security tab to submit a private vulnerability report. If private vulnerability reporting is unavailable, use a non-public contact method listed on the repository owner's GitHub profile. Include:

- the affected commit or version;
- operating system and architecture;
- reproduction steps or a minimal proof of concept;
- expected and observed impact;
- any known workaround;
- whether the report or proof contains sensitive source artwork or personal data.

Do not attach confidential artwork, credentials, access tokens, or unrelated personal data. Use a synthetic minimal fixture whenever possible.

The project targets acknowledgement within seven days. Timing for validation and a fix depends on severity and reproducibility. The maintainer will coordinate disclosure after a fix is available. A reporter who follows this policy, avoids privacy violations and service disruption, and gives the project reasonable time to respond will not face legal action from the project for good-faith research.

## Security boundaries

High-priority reports include:

- reading or writing files outside a user-selected path;
- path traversal or unsafe archive extraction;
- command execution caused by a crafted SVG, PNG, preset, or project file;
- loading external resources despite the offline guarantee;
- memory-safety defects in image processing;
- dependency or build-pipeline compromise;
- secrets committed to repository history or emitted in logs.

Crashes caused only by unsupported or malformed input are still valid bug reports, but become security issues when they cross a trust boundary, corrupt data, exhaust resources disproportionately, or enable code execution or unauthorized file access.

The implemented limits and hostile-input test surfaces are documented in [docs/security-limits.md](docs/security-limits.md).

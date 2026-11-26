# Governance

Imaguncula is a single-maintainer open source project. This document describes how decisions are made, who holds which role, and how the project continues if the maintainer becomes unavailable.

## Decision model

The project uses a maintainer-led model. The maintainer makes final decisions on scope, design, releases, and what gets merged, guided by the product principles in [PLAN.md](PLAN.md) and the published [roadmap](ROADMAP.md).

Decisions are made in public wherever possible:

- **Ideas and direction** are discussed in [GitHub Discussions](https://github.com/HusteDev/imaguncula/discussions). An idea that gains agreement becomes an issue.
- **Defects and accepted work** are tracked in [GitHub issues](https://github.com/HusteDev/imaguncula/issues).
- **Changes** reach `main` only through pull requests. A repository ruleset blocks direct pushes, force-pushes, and deletion of `main`, and requires passing code-scanning checks.
- **Significant technical decisions** are recorded as architecture decision records in [docs/adr](docs/adr/README.md), with the context, the options considered, and the consequences.
- **Security reports** are handled privately under [SECURITY.md](SECURITY.md) until a fix is available.

Anyone may comment on any proposal. When contributors disagree, the maintainer decides and explains the reasoning in the issue, pull request, or decision record.

## Roles and responsibilities

| Role | Who | Responsibilities |
| --- | --- | --- |
| Maintainer | [@HusteDev](https://github.com/HusteDev) | Sets direction and the roadmap; reviews and merges pull requests; triages issues and discussions; responds to vulnerability reports within the targets in SECURITY.md; cuts releases and keeps release automation, signing, and dependencies current; enforces the Code of Conduct; administers the repository and its secrets. |
| Contributor | Anyone who opens an issue, discussion, or pull request | Follows [CONTRIBUTING.md](CONTRIBUTING.md) and the [Code of Conduct](CODE_OF_CONDUCT.md); keeps changes focused and tested; responds to review feedback. |
| Security reporter | Anyone who reports a suspected vulnerability | Reports privately as described in SECURITY.md and allows reasonable time for a fix before disclosure. |

The maintainer is the only person with write or administrative access to the repository. New maintainers may be added by the existing maintainer after a sustained record of high-quality contributions; any change is announced in Discussions and recorded in this file.

## Continuity

The project must be able to keep creating and closing issues, accepting changes, and publishing releases within a week if the maintainer dies, is incapacitated, or stops supporting the project. To make that possible:

- The maintainer's GitHub account has a designated [account successor](https://docs.github.com/en/account-and-profile/setting-up-and-managing-your-personal-account-on-github/managing-access-to-your-personal-repositories/maintaining-ownership-continuity-of-your-personal-accounts-repositories), who can take over administration of the repository on confirmation of the maintainer's death or incapacity.
- The successor has access to the information needed to continue: the repository's deployment secrets and the accounts that publish releases.
- All release, signing, and packaging steps run from checked-in automation in `.github/workflows`, and the steps are documented in [docs/distribution.md](docs/distribution.md) and [docs/release-checklist.md](docs/release-checklist.md). Release provenance is signed with keyless Sigstore attestations, so no private signing key has to be handed over.
- The project is dual-licensed under MIT or Apache-2.0, so anyone may fork and continue it if the repository itself cannot be recovered.

## Changing this document

Changes to governance are made by pull request and announced in Discussions.

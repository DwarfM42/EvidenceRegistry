# Third-party notices — source release

## Distribution boundary

These notices accompany the EvidenceRegistry source release. They document all
**10 external packages locked in `Cargo.lock`**, deliberately including
transitive, target-specific and build-only packages even when a particular
build does not use them. This inventory is not an assertion of actual binary
linkage or of redistribution of those packages' implementation source.

The public notice assets are this Markdown inventory and
[THIRD-PARTY-NOTICES.txt](THIRD-PARTY-NOTICES.txt). They contain attribution and
original license/notice text only: **no vendored dependency source, crate
archives, Rust sysroot, executables or native runtimes**. Dependencies are
resolved separately by Cargo; archive links below identify upstream inputs,
not bundled release assets. This source release does not redistribute the
Rust sysroot or native runtimes.

## Locked dependency inventory

Names, versions and license expressions below are reproduced exactly from the
locked packages' archive manifests. Each archive was independently SHA-256
hashed before reading its contents; its digest equals the corresponding
`Cargo.lock` `checksum`. The bound `Cargo.lock` SHA-256 is
`1243b9604569c70d5ead6aa5f23e2536c8ebd30b5ae4cc2fd6239374dc2dcd64`.
This verifies consistency with the lockfile, not a separate publisher-signature
or legal-ownership certification.

| Package | Version | Declared license expression | Upstream archive | Archive SHA-256 (= Cargo.lock checksum) |
| --- | --- | --- | --- | --- |
| `block-buffer` | `0.10.4` | `MIT OR Apache-2.0` | [static.crates.io](https://static.crates.io/crates/block-buffer/block-buffer-0.10.4.crate) | `3078c7629b62d3f0439517fa394996acacc5cbc91c5a20d8c658e77abd503a71` |
| `cfg-if` | `1.0.4` | `MIT OR Apache-2.0` | [static.crates.io](https://static.crates.io/crates/cfg-if/cfg-if-1.0.4.crate) | `9330f8b2ff13f34540b44e946ef35111825727b38d33286ef986142615121801` |
| `cpufeatures` | `0.2.17` | `MIT OR Apache-2.0` | [static.crates.io](https://static.crates.io/crates/cpufeatures/cpufeatures-0.2.17.crate) | `59ed5838eebb26a2bb2e58f6d5b5316989ae9d08bab10e0e6d103e656d1b0280` |
| `crypto-common` | `0.1.7` | `MIT OR Apache-2.0` | [static.crates.io](https://static.crates.io/crates/crypto-common/crypto-common-0.1.7.crate) | `78c8292055d1c1df0cce5d180393dc8cce0abec0a7102adb6c7b1eef6016d60a` |
| `digest` | `0.10.7` | `MIT OR Apache-2.0` | [static.crates.io](https://static.crates.io/crates/digest/digest-0.10.7.crate) | `9ed9a281f7bc9b7576e61468ba615a66a5c8cfdff42420a70aa82701a3b1e292` |
| `generic-array` | `0.14.7` | `MIT` | [static.crates.io](https://static.crates.io/crates/generic-array/generic-array-0.14.7.crate) | `85649ca51fd72272d7821adaf274ad91c288277713d9c18820d8499a7ff69e9a` |
| `libc` | `0.2.189` | `MIT OR Apache-2.0` | [static.crates.io](https://static.crates.io/crates/libc/libc-0.2.189.crate) | `3eaf3ede3fee6db1a4c2ee091bf8a8b4dccdc6d17f656fb07896ee72867612f2` |
| `sha2` | `0.10.9` | `MIT OR Apache-2.0` | [static.crates.io](https://static.crates.io/crates/sha2/sha2-0.10.9.crate) | `a7507d819769d01a365ab707794a4084392c824f54a7a6a7862f8c3d0892b283` |
| `typenum` | `1.20.1` | `MIT OR Apache-2.0` | [static.crates.io](https://static.crates.io/crates/typenum/typenum-1.20.1.crate) | `b6f5e870be6c3b371b77fe0ee0bafb859fa4964b4404c27de1d380043c4dda20` |
| `version_check` | `0.9.5` | `MIT/Apache-2.0` | [static.crates.io](https://static.crates.io/crates/version_check/version_check-0.9.5.crate) | `0b928f33d975fc6ad9f86c8f283853ad26bdd5b10b7f1542aa2fa15e2289105a` |

`version_check` uses the historical literal `MIT/Apache-2.0` in its manifest.
Its packaged README states that either license may be used at the recipient's
option; its complete License section is reproduced in the text companion.
For the other dual-licensed packages, the expression remains
`MIT OR Apache-2.0`; `generic-array` declares `MIT` only. Retaining both offered
texts does **not** convert an alternative into cumulative `AND` requirements,
select an alternative for downstream recipients, or change the project's own
license.

## Original text coverage

The text companion reproduces all **20 packaged original license/notice
files**, byte-for-byte within labeled segments, plus the complete
`version_check` README License section. All archive member names were inspected
for license, notice, copyright, copying and authors materials, and all retained
original files were independently hashed and compared with the authenticated
archive members. No separate `NOTICE` file was present in these archives.
Original copyright lines, upstream spelling, formatting, license appendices
and placeholders are preserved; no generic replacement grant was substituted.

| Package/version | Complete packaged files reproduced |
| --- | --- |
| `block-buffer 0.10.4` | `LICENSE-APACHE`, `LICENSE-MIT` |
| `cfg-if 1.0.4` | `LICENSE-APACHE`, `LICENSE-MIT` |
| `cpufeatures 0.2.17` | `LICENSE-APACHE`, `LICENSE-MIT` |
| `crypto-common 0.1.7` | `LICENSE-APACHE`, `LICENSE-MIT` |
| `digest 0.10.7` | `LICENSE-APACHE`, `LICENSE-MIT` |
| `generic-array 0.14.7` | `LICENSE` |
| `libc 0.2.189` | `LICENSE-APACHE`, `LICENSE-MIT` |
| `sha2 0.10.9` | `LICENSE-APACHE`, `LICENSE-MIT` |
| `typenum 1.20.1` | `LICENSE`, `LICENSE-APACHE`, `LICENSE-MIT` |
| `version_check 0.9.5` | `LICENSE-APACHE`, `LICENSE-MIT` |

## Future binary distribution remains a separate, unresolved scope

The notice collection is complete for the locked dependency license files in
this **source-only** scope. It is not approval of an executable, toolchain or
native-runtime distribution. Before any future binary distribution:

- Determine actual linked and bundled components for the exact binary, target,
  build configuration and toolchain; do not infer them from this overinclusive
  lockfile inventory.
- Assess and satisfy applicable native library, Windows/MSVC runtime and SDK
  redistribution conditions, including any separately bundled native assets.
- Obtain and authenticate the applicable Rust sysroot originals. A prior
  toolchain inventory lacked original notices for `fortanix-sgx-abi-0.6.1`,
  `vex-sdk-0.27.1` and `wasip1-1.0.0`; their inclusion in any future binary is
  unproven. That gap remains unresolved, not waived. The prior sysroot material
  also was not independently authenticated against an official signed
  distribution manifest; none of it is reproduced in these source notices.
- Evaluate applicable obligations beyond notice reproduction, including source
  availability, modifications and other upstream conditions where relevant.

These binary/toolchain gaps are outside this source-only distribution and are
not represented as satisfied by its notice collection. Changing the release
boundary to include binaries, runtimes, archives or vendored code requires a
new distribution assessment. This document provides traceable attribution,
not a new legal grant or comprehensive legal advice.

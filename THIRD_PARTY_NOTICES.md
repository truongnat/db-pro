# DB Pro — third-party notices

The DB Pro source code is licensed under the MIT License in [`LICENSE`](LICENSE).
This file records the third-party material that is bundled or linked by the
repository. It does not replace the license text supplied by each dependency.

## Bundled font assets

The bundled Inter font files under `crates/ui/assets/fonts/` are distributed
under the SIL Open Font License, Version 1.1. The complete license text is
included at [`crates/ui/assets/fonts/OFL.txt`](crates/ui/assets/fonts/OFL.txt).

## Rust dependencies

The exact dependency versions are pinned in [`Cargo.lock`](Cargo.lock). The
license expressions for the complete resolved graph were inventoried with:

```text
cargo metadata --format-version 1 --locked
```

The resolved graph is predominantly MIT, Apache-2.0, or a choice between
those licenses. The inventory also contains dependencies with BSL-1.0,
BSD-family, CC0, CDLA-Permissive-2.0, ISC, MPL-2.0, Unicode-3.0, Zlib, and
other license expressions. Distribution tooling must preserve the applicable
copyright and license notices from those packages when producing public binary
artifacts. No dependency is relicensed by DB Pro's MIT grant.

Before a public binary release, generate and attach the complete dependency
notice bundle for the exact release `Cargo.lock`; this repository-level index
is the source inventory, not a claim that binary notice packaging is complete.

## Product identity

The DB Pro name and logos are separate from the MIT license. The MIT grant does
not grant permission to use DB Pro trademarks or imply endorsement.

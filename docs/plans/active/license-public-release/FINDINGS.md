# Findings

## F1 — P1 resolved: project license was undefined

At baseline `a81757aadcb97062c74259b83b304ede051ca5ed`, the repository had no
root `LICENSE` file and no package license metadata. This blocked public
distribution because users had no explicit source or binary redistribution
policy.

Decision: MIT License, effective for the DB Pro source repository and v0.1
artifacts, with third-party notices preserved separately and DB Pro trademarks
excluded from the MIT grant.

## F2 — P2 remains: binary notice bundle is not generated

`cargo metadata --format-version 1 --locked` exposes license expressions for
the resolved dependency graph, while the bundled Inter font already carries
`crates/ui/assets/fonts/OFL.txt`. A complete exact-artifact notice bundle still
needs to be generated before public binary distribution.

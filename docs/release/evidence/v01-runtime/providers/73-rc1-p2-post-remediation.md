# RC1 P2-H post-remediation verification (#82)

## Exact SHA

`c2ae7f63f2f8b10f9d9cad43b1df4f3dce00030a` (`#81` synthesis commit)

## Fix RC1 children — all CLOSED and ancestors of HEAD

| Issue | Representative SHA | Ancestor? |
|---|---|---|
| #238 | `a0452a32` | yes |
| #239 | `ccc0a24` | yes |
| #240 | `9e8c0f6` | yes |
| #241 | `ebf085b` | yes |
| #246 | `71ca86c` | yes |

## Affected tests re-run

```bash
cargo test -p db-pro-ui result_grid_does_not_rebuild_the_projection_every_frame --lib
cargo test -p db-pro-ui result_grid_rebuilds_the_selection_lookup_when_the_column_order_changes --lib
```

Both **ok**.

## Disposition document

`docs/release/rc1-p2-release-dispositions.md` §G lists the Fix set; Accept/Defer rows unchanged.

## Release gap statement for freeze handoff

- **P0 from Gate 5 / introspection / MySQL provider workstreams closed this session:** 0
- **Open P1 outside this P2 workstream** still exist (e.g. #145 packaged smoke, brand/license/smoke epics) and are **not** reclassified as closed by #82
- **Required RC1 P2 Fix tickets:** none open

Ready for candidate freeze workstream **#28** with the caveat that packaged-runtime smoke (#91–#95) and owner brand/license decisions remain separate gates.

# Result chart tab mounted (#248 progress)

## Claim

The existing chart engine is reachable from the query output surface: a **Chart** tab
stores `ChartConfig` on `QueryDocument` (survives tab switches) and renders through
`ChartEngine` / `ChartRenderer`.

## Landed

- `OutputTab::Chart` in query + bottom navigation panels
- `QueryDocument.chart_config`
- Type / X / Y / aggregation controls; numeric-only Y picker
- Unit tests: numeric column detection, downsample, config clone

## Not yet (issue stays open)

- Image export / copy chart
- Aggregate-from-database query generation
- 10k-row responsiveness evidence beyond `max_points` downsample unit test
- Full acceptance audit for #248

## Tests

```bash
cargo test -p db-pro-ui chart_view --lib
```

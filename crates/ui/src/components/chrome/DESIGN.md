# Chrome design

Avatar, Skeleton, EmptyState, and Toolbar are passive composition pieces. Avatar resolves initials before an optional icon, paints a semantic status dot, and exposes an explicit accessible label or a text fallback. Skeleton allocates one placeholder and may paint a clipped moving band. EmptyState wraps its description and returns only the optional action button response. Toolbar supplies a compact themed frame and delegates actions to shared Button widgets.

Skeleton uses `DbProTheme::reduce_motion`: pulse and shimmer stop, the placeholder uses a steady mid-range alpha, and it no longer requests continuous repaint. Its `shimmer(false)` option only suppresses the moving band; the existing pulse remains unless reduced motion is enabled. Shared radius and stroke tokens are read directly from `tokens.rs`.

Avatar cost is fixed. EmptyState and Toolbar cost follows their child content. Skeleton performs constant painting work and schedules repaint only while motion is enabled. Long descriptions wrap to the parent width; the parent remains responsible for scrolling.

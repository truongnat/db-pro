# DevTools design

`TerminalBlock` renders a terminal-like passive surface: a fixed header with three decorative window dots and a title, followed by the caller's output text. `ProgressRing` sanitizes progress and radius in handlers, allocates a square, and paints the track plus a fixed-step arc. It is a visual indicator and does not start work or animate itself.

The terminal dots are decorative and the output is ordinary text; callers should provide surrounding labels when the context is not clear. `ProgressRing` reports a progress-bar role/value through egui metadata. Shared colors come from `DbProTheme`; component config owns terminal and ring dimensions.

Terminal layout cost follows the output text length. Progress-ring painting uses a fixed 32-point arc approximation, so per-frame cost is constant. There is no automatic repaint or motion; the parent controls when updated progress is shown. Long output is not virtualized and should be placed in a bounded scrolling parent when needed.

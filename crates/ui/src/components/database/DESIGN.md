# Database component design

`mod.rs` exports `ConnectionCard`, `DatabaseTypeBadge`, `ConnectionStatus`, `DatabaseDriver`, and `ConnectionCardAction`. `ui.rs` measures and paints the card/badge, `handler.rs` maps provider/status to labels, colors and available primary actions, and `config.rs` holds badge/status geometry. Theme colors, typography and shared spacing use `DbProTheme` and shared tokens.

## Event flow

The caller provides a snapshot of connection identity and status each frame. The handler derives one status presentation and the eligible primary control: Connect for Disconnected, Retry for Error (emits the existing Connect action), disabled Connecting... while Connecting, or Disconnect while Connected. Edit is always available. A click returns an action; connection work and state refresh remain in the caller/runtime. The card itself never mutates or validates connection state.

## Rendering cost and content

The card paints one frame, identity/host labels, status and a small fixed set of buttons. Long name, database, and host labels are truncated to their allotted widths with the full value available on hover. The identity column yields space to the status column, so narrow cards can leave little room for text. The provider badge is a fixed-size, non-interactive surface. Empty/loading/error states are represented by caller-supplied values and status, not fetched by this component.

## Accessibility and theme

Status uses text plus a colored indicator; connecting controls are disabled, and provider/SSL badges expose label metadata. Buttons use the shared accessible Button widget. Both light/dark semantic colors come from the theme. Host and identity remain discoverable on hover when visually truncated.

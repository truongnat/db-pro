# Findings

- The native panel was presentation-only: its input was disabled and it had no message state or command path.
- The existing frontend has template-agent behavior, but native schema DTOs currently expose only summary table and column names. The native first slice therefore generates safe, inspectable drafts from that summary and does not pretend to have foreign-key context it does not receive.
- Native architecture forbids putting credentials in UI state. Provider configuration and network calls remain a later runtime concern.

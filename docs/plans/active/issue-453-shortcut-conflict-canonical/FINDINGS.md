# Issue #453 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/453
- Fix: KeybindingSettings::conflicts parses shortcut parts and groups by canonical (Key, want_mod, want_shift, want_alt) tuple so alias variations (ctrl vs mod vs cmd) detect conflicts properly.

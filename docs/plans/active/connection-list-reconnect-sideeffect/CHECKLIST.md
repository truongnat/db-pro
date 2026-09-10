# Checklist — QA-P1-09 Connection-list query reconnect side-effect

- [ ] PLAN.md, CHECKLIST.md, FINDINGS.md, VERIFICATION.md created
- [ ] `useConnectionList` queryFn does not trigger `restoreSession` on generic refetches
- [ ] One-shot session restoration logic is cleanly separated
- [ ] Frontend unit tests verify refetch safety
- [ ] Quality gates (FE + Rust) executed and passing
- [ ] PR published against main

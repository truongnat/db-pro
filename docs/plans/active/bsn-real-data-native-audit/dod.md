# Definition of Done — BSN Real-Data Findings Remediation

- Current source triage distinguishes confirmed defects from historical or unverified visual/focus candidates.
- Confirmed correctness defects have consumer-visible failure-injection regression tests and isolated provider verification.
- Native i18n catalogs load all tested entries across five supported languages, retain both diagram key groups, and preserve interpolation placeholders.
- Confirmed missing high-impact translations and malformed Korean export wording are corrected.
- Any visual fixes use current native capture evidence; synthetic fixtures are labeled and never represented as BSN output.
- BSN remains read-only; no customer values, credentials, endpoint data, or raw query contents enter artifacts.
- Focused tests and relevant build/quality gates are executed and reported exactly.
- Native BSN screenshot/focus limitations remain explicit; plan is not moved to COMPLETED while those runtime gates are open.
- Evidence handoff follows the canonical template and ends with a Vietnamese summary.

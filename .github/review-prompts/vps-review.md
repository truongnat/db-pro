Review the current Pull Request.

Compare:

origin/${BASE_REF}...HEAD

Read:
- AGENTS.md
- REVIEW.md
- the complete PR diff
- relevant surrounding implementation
- relevant tests

Do not modify anything.
Do not post to GitHub yourself.

For every P0/P1 finding include:
- exact file/function evidence
- a concrete failure scenario
- severity justification

Pay special attention to:
- SQL safety
- transaction atomicity and rollback
- PostgreSQL/SQLite divergence
- stale metadata/cache/UI state
- precision and NULL semantics
- capability mismatches
- missing regression tests

Source reasoning is not runtime verification.

Produce the final review as Markdown in your response.

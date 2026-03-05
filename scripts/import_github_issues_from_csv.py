#!/usr/bin/env python3
"""Import GitHub issues from a CSV file using `gh issue create`.

Default behavior is dry-run. Use --apply to create issues.
"""

from __future__ import annotations

import argparse
import csv
import os
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


DEFAULT_CSV = "docs/issues/mvp/import/github-import.csv"
PLACEHOLDER_ACCOUNT = "<ACCOUNT_NAME>"
PLACEHOLDER_REPO = "<REPO_NAME>"


@dataclass
class IssueRow:
    line: int
    title: str
    body: str
    labels: list[str]
    status: str
    account_name: str
    repo_name: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Import GitHub issues from CSV via gh CLI."
    )
    parser.add_argument(
        "--csv",
        default=DEFAULT_CSV,
        help=f"Path to CSV file (default: {DEFAULT_CSV})",
    )
    parser.add_argument(
        "--repo",
        default="",
        help="Target repo in owner/name format. If omitted, uses GH_REPO or CSV columns.",
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="Create issues for real. Without this flag, script runs dry-run only.",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=0,
        help="Only process first N eligible issues (0 means all).",
    )
    return parser.parse_args()


def ensure_gh_cli() -> None:
    if shutil.which("gh") is None:
        raise RuntimeError("`gh` CLI not found. Install GitHub CLI first.")


def load_rows(path: Path) -> list[IssueRow]:
    if not path.exists():
        raise RuntimeError(f"CSV file not found: {path}")

    rows: list[IssueRow] = []
    with path.open("r", encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle)
        for index, row in enumerate(reader, start=2):
            title = (row.get("title") or "").strip()
            if not title:
                continue

            labels = [
                item.strip()
                for item in (row.get("labels") or "").split(",")
                if item.strip()
            ]

            rows.append(
                IssueRow(
                    line=index,
                    title=title,
                    body=row.get("body") or "",
                    labels=labels,
                    status=(row.get("status") or "").strip().lower(),
                    account_name=(row.get("account_name") or "").strip(),
                    repo_name=(row.get("repo_name") or "").strip(),
                )
            )
    return rows


def resolve_repo(flag_repo: str, rows: Iterable[IssueRow]) -> str:
    if flag_repo.strip():
        return flag_repo.strip()

    gh_repo = (os.environ.get("GH_REPO") or "").strip()
    if gh_repo:
        return gh_repo

    for row in rows:
        if (
            row.account_name
            and row.repo_name
            and row.account_name != PLACEHOLDER_ACCOUNT
            and row.repo_name != PLACEHOLDER_REPO
        ):
            return f"{row.account_name}/{row.repo_name}"

    raise RuntimeError(
        "Unable to resolve repo. Pass --repo owner/name or set GH_REPO "
        "or replace placeholders in CSV."
    )


def eligible_rows(rows: list[IssueRow], limit: int) -> list[IssueRow]:
    filtered = [row for row in rows if row.status in {"", "open"}]
    if limit > 0:
        return filtered[:limit]
    return filtered


def run_dry(repo: str, rows: list[IssueRow]) -> None:
    print(f"[dry-run] repo={repo} issue_count={len(rows)}")
    for row in rows:
        labels = ",".join(row.labels) if row.labels else "(none)"
        print(f"- line={row.line} title={row.title!r} labels={labels}")


def create_issue(repo: str, row: IssueRow) -> str:
    command = [
        "gh",
        "issue",
        "create",
        "--repo",
        repo,
        "--title",
        row.title,
        "--body",
        row.body,
    ]

    for label in row.labels:
        command.extend(["--label", label])

    result = subprocess.run(command, check=True, capture_output=True, text=True)
    return result.stdout.strip()


def run_apply(repo: str, rows: list[IssueRow]) -> int:
    print(f"[apply] repo={repo} issue_count={len(rows)}")
    created = 0
    for row in rows:
        try:
            url = create_issue(repo, row)
            created += 1
            print(f"- created line={row.line} url={url}")
        except subprocess.CalledProcessError as error:
            stderr = (error.stderr or "").strip()
            print(
                f"- failed line={row.line} title={row.title!r} error={stderr}",
                file=sys.stderr,
            )
            return 1
    print(f"[done] created={created}")
    return 0


def main() -> int:
    args = parse_args()

    try:
        ensure_gh_cli()
        csv_path = Path(args.csv).resolve()
        rows = load_rows(csv_path)
        if not rows:
            raise RuntimeError(f"No issue rows found in CSV: {csv_path}")

        repo = resolve_repo(args.repo, rows)
        targets = eligible_rows(rows, args.limit)
        if not targets:
            print("No eligible rows to process (status must be open or empty).")
            return 0

        if args.apply:
            return run_apply(repo, targets)

        run_dry(repo, targets)
        print("Dry-run only. Re-run with --apply to create issues.")
        return 0
    except RuntimeError as error:
        print(str(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())

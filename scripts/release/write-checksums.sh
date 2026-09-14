#!/usr/bin/env bash
#
# Generates SHA256SUMS.txt covering exactly the three v0.1 release archives.
#
# Usage: write-checksums.sh <version> <artifacts-dir> [output-file]
#
# The checksum file is written in coreutils format ("<hash>  <name>", names
# relative to the artifacts directory) so it verifies with:
#
#   cd <artifacts-dir> && sha256sum -c SHA256SUMS.txt     (GNU coreutils)
#   cd <artifacts-dir> && shasum -a 256 -c SHA256SUMS.txt (macOS/perl)
#
# The contents of SHA256SUMS.txt are the only thing written to stdout; progress
# and warnings go to stderr.
set -euo pipefail

usage() {
    echo "usage: $(basename "$0") <version> <artifacts-dir> [output-file]" >&2
}

if [[ $# -lt 2 || $# -gt 3 ]]; then
    usage
    exit 2
fi

version="$1"
artifacts_dir="$2"
output_file="${3:-${artifacts_dir}/SHA256SUMS.txt}"

if [[ ! "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.]+)?$ ]]; then
    echo "::error::invalid version '${version}'" >&2
    exit 1
fi
if [[ ! -d "${artifacts_dir}" ]]; then
    echo "::error::artifacts directory not found: ${artifacts_dir}" >&2
    exit 1
fi

# Exactly the v0.1 release contract: three portable archives, in a stable order.
archives=(
    "db-pro-v${version}-linux-x86_64.tar.gz"
    "db-pro-v${version}-macos-arm64.tar.gz"
    "db-pro-v${version}-windows-x86_64.zip"
)

for name in "${archives[@]}"; do
    if [[ ! -f "${artifacts_dir}/${name}" ]]; then
        echo "::error::expected release archive is missing: ${artifacts_dir}/${name}" >&2
        exit 1
    fi
done

if command -v sha256sum >/dev/null 2>&1; then
    hasher=(sha256sum)
elif command -v shasum >/dev/null 2>&1; then
    hasher=(shasum -a 256)
else
    echo "::error::neither sha256sum nor shasum is available" >&2
    exit 1
fi

for path in "${artifacts_dir}"/db-pro-*; do
    [[ -e "${path}" ]] || continue
    base="$(basename "${path}")"
    expected=0
    for name in "${archives[@]}"; do
        if [[ "${base}" == "${name}" ]]; then
            expected=1
        fi
    done
    if [[ "${expected}" -eq 0 ]]; then
        echo "::warning::not part of the v0.1 checksum set, ignoring: ${base}" >&2
    fi
done

tmp_file="${output_file}.tmp"
(
    cd "${artifacts_dir}"
    for name in "${archives[@]}"; do
        "${hasher[@]}" "${name}"
    done
) > "${tmp_file}"
mv "${tmp_file}" "${output_file}"

entry_count="$(wc -l < "${output_file}" | tr -d '[:space:]')"
if [[ "${entry_count}" != "3" ]]; then
    echo "::error::expected 3 checksum entries, found ${entry_count}" >&2
    exit 1
fi

echo "Wrote ${output_file} (${entry_count} entries, ${hasher[0]})" >&2
cat "${output_file}"

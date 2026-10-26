#!/usr/bin/env bash
#
# Builds the v0.1 Linux release archive: the native binary, the shared install
# note and the license/notices files, wrapped in
# db-pro-v<version>-linux-x86_64.tar.gz.
#
# Usage: package-linux.sh <version> <binary-path> <output-dir>
#
# Contract (goal-3 sections 9 and 10, V01-06 step 2):
#
#   db-pro-native
#   README-INSTALL.txt
#   LICENSE
#   THIRD_PARTY_NOTICES.md
#
# The artifact is a portable tar.gz: no .deb/.rpm/AppImage is part of the v0.1
# contract, and the archive is unsigned.
#
# Progress goes to stderr; the absolute path of the created archive is the only
# line written to stdout, so callers can capture it with command substitution.
set -euo pipefail

usage() {
    echo "usage: $(basename "$0") <version> <binary-path> <output-dir>" >&2
}

if [[ $# -ne 3 ]]; then
    usage
    exit 2
fi

version="$1"
binary="$2"
output_dir="$3"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
install_note="${script_dir}/README-INSTALL.txt"
repo_root="$(cd "${script_dir}/../.." && pwd)"
license_file="${repo_root}/LICENSE"
notices_file="${repo_root}/THIRD_PARTY_NOTICES.md"

if [[ ! "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.]+)?$ ]]; then
    echo "::error::invalid version '${version}'" >&2
    exit 1
fi
if [[ ! -f "${binary}" ]]; then
    echo "::error::binary not found: ${binary}" >&2
    exit 1
fi
if [[ ! -f "${install_note}" ]]; then
    echo "::error::install note not found: ${install_note}" >&2
    exit 1
fi
if [[ ! -f "${license_file}" ]]; then
    echo "::error::license file not found: ${license_file}" >&2
    exit 1
fi
if [[ ! -f "${notices_file}" ]]; then
    echo "::error::third-party notices not found: ${notices_file}" >&2
    exit 1
fi

staging="$(mktemp -d "${TMPDIR:-/tmp}/db-pro-package.XXXXXX")"
cleanup() {
    rm -rf "${staging}"
}
trap cleanup EXIT

cp "${binary}" "${staging}/db-pro-native"
chmod 755 "${staging}/db-pro-native"
cp "${install_note}" "${staging}/README-INSTALL.txt"
cp "${license_file}" "${staging}/LICENSE"
cp "${notices_file}" "${staging}/THIRD_PARTY_NOTICES.md"

mkdir -p "${output_dir}"
output_dir="$(cd "${output_dir}" && pwd)"
archive="${output_dir}/db-pro-v${version}-linux-x86_64.tar.gz"
rm -f "${archive}"

members="db-pro-native README-INSTALL.txt LICENSE THIRD_PARTY_NOTICES.md"

# GNU tar can produce a deterministic member order/metadata; bsdtar (used when
# this script is exercised on macOS for layout testing) cannot, so fall back to
# the plain invocation there. Both invocations are functional.
if tar --version 2>/dev/null | head -n 1 | grep -q "GNU tar"; then
    # shellcheck disable=SC2086  # member list is a deliberate word-split table
    tar -czf "${archive}" \
        --sort=name --owner=0 --group=0 --numeric-owner --mtime=@0 \
        -C "${staging}" ${members}
else
    echo "NOTE: GNU tar not detected, writing a non-canonical archive" >&2
    # shellcheck disable=SC2086  # member list is a deliberate word-split table
    tar -czf "${archive}" -C "${staging}" ${members}
fi

# The archive must contain exactly the contract layout and nothing else: no
# build output, no user state, no extra license text beyond LICENSE and
# THIRD_PARTY_NOTICES.md.
listing="$(tar -tzf "${archive}")"
while IFS= read -r entry; do
    case "${entry}" in
        "db-pro-native" | "README-INSTALL.txt" | "LICENSE" | "THIRD_PARTY_NOTICES.md")
            ;;
        *)
            echo "::error::unexpected archive entry: ${entry}" >&2
            exit 1
            ;;
    esac
done <<< "${listing}"

for required in \
    "db-pro-native" \
    "README-INSTALL.txt" \
    "LICENSE" \
    "THIRD_PARTY_NOTICES.md"
do
    if ! grep -qx -F "${required}" <<< "${listing}"; then
        echo "::error::archive entry is missing: ${required}" >&2
        exit 1
    fi
done

echo "Archive entries:" >&2
echo "${listing}" | sed 's/^/  /' >&2
echo "Archive: $(ls -l "${archive}" | awk '{ print $5" bytes" }')" >&2
if command -v sha256sum >/dev/null 2>&1; then
    echo "SHA256:  $(sha256sum "${archive}" | awk '{ print $1 }')" >&2
else
    echo "SHA256:  $(shasum -a 256 "${archive}" | awk '{ print $1 }')" >&2
fi

printf '%s\n' "${archive}"

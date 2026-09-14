#!/usr/bin/env bash
#
# Builds the v0.1 macOS release archive: a minimal "DB Pro.app" bundle plus the
# shared install note, wrapped in db-pro-v<version>-macos-arm64.tar.gz.
#
# Usage: package-macos.sh <version> <binary-path> <output-dir>
#
# Contract (goal-3 sections 7 and 10, V01-06 step 2):
#
#   DB Pro.app/Contents/MacOS/db-pro-native
#   DB Pro.app/Contents/Info.plist
#   README-INSTALL.txt
#
# macOS-only by design (plutil, otool, bsdtar's --no-mac-metadata). The archive
# is unsigned and not notarized: signing is deliberately out of the v0.1
# contract and no signing secrets exist.
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

staging="$(mktemp -d "${TMPDIR:-/tmp}/db-pro-package.XXXXXX")"
cleanup() {
    rm -rf "${staging}"
}
trap cleanup EXIT

app="${staging}/DB Pro.app"
macos_dir="${app}/Contents/MacOS"
mkdir -p "${macos_dir}"
cp "${binary}" "${macos_dir}/db-pro-native"
chmod 755 "${macos_dir}/db-pro-native"

# Minimum supported OS is measured from the binary rather than assumed, so the
# bundle metadata cannot drift from what was actually compiled.
min_os="$(otool -l "${binary}" | awk '/LC_BUILD_VERSION/ { in_build = 1 } in_build && $1 == "minos" { print $2; exit }')"
if [[ -z "${min_os}" ]]; then
    echo "::error::could not derive LSMinimumSystemVersion from LC_BUILD_VERSION in ${binary}" >&2
    exit 1
fi
echo "Measured LC_BUILD_VERSION minos: ${min_os}" >&2

cat > "${app}/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleName</key>
	<string>DB Pro</string>
	<key>CFBundleDisplayName</key>
	<string>DB Pro</string>
	<key>CFBundleIdentifier</key>
	<string>com.dbpro.app</string>
	<key>CFBundleExecutable</key>
	<string>db-pro-native</string>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
	<key>CFBundleShortVersionString</key>
	<string>${version}</string>
	<key>CFBundleVersion</key>
	<string>${version}</string>
	<key>CFBundleInfoDictionaryVersion</key>
	<string>6.0</string>
	<key>LSMinimumSystemVersion</key>
	<string>${min_os}</string>
</dict>
</plist>
PLIST

plutil -lint "${app}/Contents/Info.plist" >&2

cp "${install_note}" "${staging}/README-INSTALL.txt"

mkdir -p "${output_dir}"
output_dir="$(cd "${output_dir}" && pwd)"
archive="${output_dir}/db-pro-v${version}-macos-arm64.tar.gz"
rm -f "${archive}"

# COPYFILE_DISABLE stops AppleDouble ("._") sidecars; --no-xattrs stops macOS
# extended attributes (com.apple.provenance, com.apple.macl, quarantine) from
# being stored as pax headers inside the release archive.
(
    cd "${staging}"
    COPYFILE_DISABLE=1 tar --no-xattrs -czf "${archive}" "DB Pro.app" "README-INSTALL.txt"
)

# The archive must contain exactly the contract layout and nothing else: no
# LICENSE (the project has no license decision yet), no user state, no build
# output.
listing="$(tar -tzf "${archive}")"
while IFS= read -r entry; do
    case "${entry}" in
        "README-INSTALL.txt" \
        | "DB Pro.app/" \
        | "DB Pro.app/Contents/" \
        | "DB Pro.app/Contents/Info.plist" \
        | "DB Pro.app/Contents/MacOS/" \
        | "DB Pro.app/Contents/MacOS/db-pro-native")
            ;;
        *)
            echo "::error::unexpected archive entry: ${entry}" >&2
            exit 1
            ;;
    esac
done <<< "${listing}"

for required in \
    "DB Pro.app/Contents/Info.plist" \
    "DB Pro.app/Contents/MacOS/db-pro-native" \
    "README-INSTALL.txt"
do
    if ! grep -qx -F "${required}" <<< "${listing}"; then
        echo "::error::archive entry is missing: ${required}" >&2
        exit 1
    fi
done

echo "Archive entries:" >&2
echo "${listing}" | sed 's/^/  /' >&2
echo "Archive: $(ls -l "${archive}" | awk '{ print $5" bytes" }')" >&2
echo "SHA256:  $(shasum -a 256 "${archive}" | awk '{ print $1 }')" >&2

printf '%s\n' "${archive}"

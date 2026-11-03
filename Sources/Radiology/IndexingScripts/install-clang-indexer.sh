#!/bin/sh
set -eu

tool_bin=$1
tool_version=$2
expected_digest=$3
asset=${4:-scip-clang-arm64-darwin}
curl_program=${5:-/usr/bin/curl}
sha256_program=${6:-}
mkdir -p "$tool_bin"
staging=$(mktemp "$tool_bin/.scip-clang.XXXXXX")
trap 'rm -f "$staging"' EXIT HUP INT TERM

echo "Downloading scip-clang $tool_version ($asset)…"
"$curl_program" --fail --location --retry 2 --silent --show-error \
    "https://github.com/sourcegraph/scip-clang/releases/download/v$tool_version/$asset" -o "$staging"
if [ -n "$sha256_program" ]; then
    actual_digest=$("$sha256_program" < "$staging")
else
    actual_digest=$(/usr/bin/shasum -a 256 "$staging")
fi
[ "${actual_digest%% *}" = "$expected_digest" ] || { echo 'scip-clang download checksum mismatch.' >&2; exit 1; }
chmod 755 "$staging"
"$staging" --version
mv -f "$staging" "$tool_bin/scip-clang"
echo 'C/C++ semantic indexer installed.'

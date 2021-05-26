#!/usr/bin/env sh

if [ $# -lt 2 ]; then
  echo "Usage: $0 [changelog file] [version] "
  exit 1
fi

VERSION=$2
CHANGELOG=$1

tail +3 "$CHANGELOG" >tmpfile &&
  $(dirname "${BASH_SOURCE[0]}")/generate_changelog.sh "$VERSION" | cat - tmpfile >"$CHANGELOG"
rm tmpfile

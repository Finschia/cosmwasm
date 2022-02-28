#!/usr/bin/env sh

if [ $# -lt 1 ]; then
  echo "Usage: $0 [version]"
  exit 1
fi

VERSION=$1
CMD='git-chglog'

if ! command -v git-chglog >/dev/null 2>&1; then
  if command -v docker >/dev/null 2>&1; then
    CMD="docker run -v $(pwd):/workdir quay.io/git-chglog/git-chglog"
  else
    echo "git-chglog or docker required"
    exit 2
  fi
fi

if [ -z "$(git tag -l "$VERSION")" ]; then
  $CMD --next-tag "$VERSION" "$VERSION"
else
  $CMD "$VERSION"
fi

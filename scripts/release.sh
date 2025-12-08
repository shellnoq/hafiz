#!/bin/bash
set -e

# Usage: ./scripts/release.sh [patch|minor|major]

TYPE=${1:-patch}

# Get current version
CURRENT=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
IFS='.' read -ra PARTS <<< "$CURRENT"

MAJOR=${PARTS[0]}
MINOR=${PARTS[1]}
PATCH=${PARTS[2]}

case $TYPE in
  patch)
    PATCH=$((PATCH + 1))
    ;;
  minor)
    MINOR=$((MINOR + 1))
    PATCH=0
    ;;
  major)
    MAJOR=$((MAJOR + 1))
    MINOR=0
    PATCH=0
    ;;
  *)
    echo "Usage: $0 [patch|minor|major]"
    exit 1
    ;;
esac

NEW_VERSION="$MAJOR.$MINOR.$PATCH"

echo "Current version: $CURRENT"
echo "New version: $NEW_VERSION"
echo ""
read -p "Continue? [y/N] " -n 1 -r
echo

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo "Aborted."
  exit 1
fi

# Update Cargo.toml
sed -i "s/^version = \"$CURRENT\"/version = \"$NEW_VERSION\"/" Cargo.toml

# Update workspace members
for crate in crates/*/Cargo.toml; do
  sed -i "s/^version = \"$CURRENT\"/version = \"$NEW_VERSION\"/" "$crate"
done

# Commit
git add -A
git commit -m "chore: release v$NEW_VERSION"

# Tag
git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"

echo ""
echo "Created release v$NEW_VERSION"
echo "Run 'git push origin main --tags' to publish"

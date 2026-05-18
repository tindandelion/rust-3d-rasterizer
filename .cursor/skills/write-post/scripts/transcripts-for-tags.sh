#!/usr/bin/env bash
# List Cursor agent-transcript files whose creation or modification time falls
# within the time window defined by one or more Git tags.
#
# Examples:
#   .cursor/skills/write-post/scripts/transcripts-for-tags.sh 0.0.6
#   .cursor/skills/write-post/scripts/transcripts-for-tags.sh 0.0.5 0.0.6
#
# Configuration (from .env.local at the repo root):
#   REPO              Git repo containing version tags (rust-3d-rasterizer main checkout)
#   TRANSCRIPTS_DIR   Cursor agent-transcripts directory

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
ENV_FILE="$REPO_ROOT/.env.local"

load_env_local() {
  if [[ -f "$ENV_FILE" ]]; then
    set -a
    # shellcheck source=/dev/null
    source "$ENV_FILE"
    set +a
  fi
}

load_env_local

usage() {
  cat <<'EOF'
Usage: transcripts-for-tags.sh [OPTIONS] TAG [TAG ...]

Resolve Git tag timestamps and list agent-transcript files created or modified
in the resulting time window.

Time window:
  • One tag: from the previous version tag (by sort -V) through this tag (inclusive).
  • Two or more tags: from the earliest tag time through the latest (inclusive).

Options:
  -r, --repo PATH           Git repository with the tags (overrides REPO)
  -t, --transcripts-dir DIR Agent transcripts root (overrides TRANSCRIPTS_DIR)
  -q, --quiet               Print paths only (no header or tag summary)
  -h, --help                Show this help

Configuration (.env.local at repo root):
  REPO              Git checkout with version tags (required unless -r is passed)
  TRANSCRIPTS_DIR   Cursor agent-transcripts directory (required unless -t is passed)
EOF
}

repo=""
transcripts_dir="${TRANSCRIPTS_DIR:-}"
quiet=0
tags=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    -r|--repo)
      repo="${2:?missing argument for $1}"
      shift 2
      ;;
    -t|--transcripts-dir)
      transcripts_dir="${2:?missing argument for $1}"
      shift 2
      ;;
    -q|--quiet)
      quiet=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --)
      shift
      while [[ $# -gt 0 ]]; do tags+=("$1"); shift; done
      ;;
    -*)
      echo "transcripts-for-tags.sh: unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
    *)
      tags+=("$1")
      shift
      ;;
  esac
done

if [[ ${#tags[@]} -eq 0 ]]; then
  echo "transcripts-for-tags.sh: at least one TAG is required" >&2
  usage >&2
  exit 2
fi

if [[ -z "$repo" ]]; then
  if [[ -n "${REPO:-}" ]]; then
    repo="$REPO"
  else
    echo "transcripts-for-tags.sh: REPO is not set" >&2
    echo "Set it in $ENV_FILE or pass --repo" >&2
    exit 2
  fi
fi

if [[ ! -d "$repo/.git" && ! -f "$repo/.git" ]]; then
  echo "transcripts-for-tags.sh: not a Git repository: $repo" >&2
  exit 2
fi

if [[ -z "$transcripts_dir" ]]; then
  echo "transcripts-for-tags.sh: TRANSCRIPTS_DIR is not set" >&2
  echo "Set it in $ENV_FILE or pass --transcripts-dir" >&2
  exit 2
fi

if [[ ! -d "$transcripts_dir" ]]; then
  echo "transcripts-for-tags.sh: transcripts directory not found: $transcripts_dir" >&2
  exit 2
fi

# Epoch seconds for a tag (annotated or lightweight).
tag_epoch() {
  local tag="$1"
  local epoch
  epoch="$(git -C "$repo" log -1 --format='%ct' "$tag" 2>/dev/null)" || {
    echo "transcripts-for-tags.sh: unknown or invalid tag: $tag" >&2
    exit 2
  }
  printf '%s' "$epoch"
}

# ISO-8601 local time for display.
epoch_to_iso() {
  date -d "@$1" '+%Y-%m-%d %H:%M:%S %z' 2>/dev/null \
    || date -r "$1" '+%Y-%m-%d %H:%M:%S %z'
}

# Previous semver tag before REF (tags only, sorted -V).
previous_tag() {
  local ref="$1"
  git -C "$repo" tag -l '[0-9]*.[0-9]*.[0-9]*' --sort=-version:refname \
    | awk -v ref="$ref" '$0 == ref { if (getline) print; exit }'
}

declare -a tag_epochs=()
for t in "${tags[@]}"; do
  tag_epochs+=("$(tag_epoch "$t")")
done

window_start=""
window_end=""

if [[ ${#tags[@]} -eq 1 ]]; then
  end="${tag_epochs[0]}"
  prev="$(previous_tag "${tags[0]}")" || true
  if [[ -n "$prev" ]]; then
    window_start="$(tag_epoch "$prev")"
    window_end="$end"
    window_label="(${prev} → ${tags[0]}]"
  else
    window_start="$end"
    window_end="$end"
    window_label="(tag ${tags[0]} only — no earlier semver tag)"
  fi
else
  window_start="${tag_epochs[0]}"
  window_end="${tag_epochs[0]}"
  for e in "${tag_epochs[@]}"; do
    (( e < window_start )) && window_start="$e"
    (( e > window_end )) && window_end="$e"
  done
  window_label="(min tag → max tag: ${tags[*]})"
fi

# find -newermt uses strict 'after'; subtract 1s so the boundary tag time is included.
start_iso="$(epoch_to_iso "$window_start")"
end_iso="$(epoch_to_iso "$window_end")"

# Best-effort birth time (macOS: stat -f %B; GNU: stat -c %W, may be 0).
file_activity_epoch() {
  local f="$1"
  local m b
  m=$(stat -c '%Y' "$f" 2>/dev/null) || m=$(stat -f '%m' "$f")
  b=$(stat -c '%W' "$f" 2>/dev/null) || b=$(stat -f '%B' "$f" 2>/dev/null || echo 0)
  if [[ "$b" =~ ^[0-9]+$ ]] && (( b > 0 )) && (( b > m )); then
    printf '%s' "$b"
  else
    printf '%s' "$m"
  fi
}

in_window() {
  local epoch="$1"
  (( epoch >= window_start && epoch <= window_end ))
}

matches=()
while IFS= read -r -d '' f; do
  activity="$(file_activity_epoch "$f")"
  if in_window "$activity"; then
    matches+=("$f")
  fi
done < <(find "$transcripts_dir" -type f -print0 2>/dev/null)

# Sort by activity time, then path.
sorted_lines=()
for f in "${matches[@]}"; do
  sorted_lines+=("$(file_activity_epoch "$f")	$f")
done
IFS=$'\n'
sorted_lines=($(printf '%s\n' "${sorted_lines[@]:-}" | sort -n))
unset IFS

if (( quiet == 0 )); then
  echo "Repository:        $repo"
  echo "Transcripts:       $transcripts_dir"
  echo "Tags:              ${tags[*]}"
  echo "Window $window_label"
  echo "  from $start_iso"
  echo "  to   $end_iso"
  echo "Files (${#sorted_lines[@]}):"
fi

for line in "${sorted_lines[@]}"; do
  [[ -z "$line" ]] && continue
  epoch="${line%%	*}"
  path="${line#*	}"
  if (( quiet == 0 )); then
    printf '  %s  %s\n' "$(epoch_to_iso "$epoch")" "$path"
  else
    printf '%s\n' "$path"
  fi
done

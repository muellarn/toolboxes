#!/usr/bin/env bash
# Back up both toolboxes: copy the Python tool venv requirements.txt into this
# repo, then commit and push all changes. Usage: backup.sh ["commit message"]
# VENV_TOOLS overrides the venv location (default: %USERPROFILE%\.venv-tools).
set -euo pipefail
cd "$(dirname "$(readlink -f "$0")")"
if [ -z "${VENV_TOOLS:-}" ]; then
  win_home=$(cd /mnt/c && cmd.exe /c "echo %USERPROFILE%" 2>/dev/null | tr -d "\r")
  VENV_TOOLS="$(wslpath -u "$win_home")/.venv-tools"
fi
mkdir -p python
cp "$VENV_TOOLS/requirements.txt" python/requirements.txt
git add -A
if git diff --cached --quiet; then echo "nothing to back up"; exit 0; fi
git commit -q -m "${1:-chore: update toolboxes}"
git push -q
echo "backed up: $(git log -1 --format=%s)"

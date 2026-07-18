#!/usr/bin/env bash
# Inicializa el repo local y lo publica en GitHub.

set -Eeuo pipefail

REMOTE_URL="${1:-https://github.com/isgaar/Honey.git}"
COMMIT_MESSAGE="${HONEY_COMMIT_MESSAGE:-first commit}"

if ! command -v git >/dev/null 2>&1; then
    printf 'ERROR git no esta instalado\n' >&2
    exit 1
fi

git init
git add .

if git diff --cached --quiet; then
    printf 'No hay cambios nuevos para commitear.\n'
else
    git commit -m "$COMMIT_MESSAGE"
fi

git branch -M main

if git remote get-url origin >/dev/null 2>&1; then
    git remote set-url origin "$REMOTE_URL"
else
    git remote add origin "$REMOTE_URL"
fi

git push -u origin main


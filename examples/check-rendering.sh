#!/usr/bin/env bash
# Ejemplo de diagnostico para ver como resuelve fuentes el sistema.

set -Eeuo pipefail

if ! command -v fc-match >/dev/null 2>&1; then
    printf 'fc-match no esta disponible\n' >&2
    exit 1
fi

for family in sans-serif serif monospace emoji; do
    printf '%-12s %s\n' "$family" "$(fc-match "$family" | head -n 1)"
done


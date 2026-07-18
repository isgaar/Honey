#!/usr/bin/env bash
# Honey installer
# Instala el runtime en modo usuario, sin sudo y sin modificar archivos del sistema.

set -Eeuo pipefail

APP_NAME="Honey"
APP_VERSION="0.1.0"
SOURCE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PREFIX="${HONEY_PREFIX:-$HOME/.local}"
INSTALL_DIR="${HONEY_HOME:-$HOME/.local/share/honey}"
BIN_DIR="$PREFIX/bin"
BIN_PATH="$BIN_DIR/honey"

green='\033[0;32m'
yellow='\033[1;33m'
red='\033[0;31m'
reset='\033[0m'

ok() { printf '%bOK%b %s\n' "$green" "$reset" "$1"; }
warn() { printf '%bWARN%b %s\n' "$yellow" "$reset" "$1"; }
fail() { printf '%bERROR%b %s\n' "$red" "$reset" "$1" >&2; exit 1; }

usage() {
    cat <<'EOF'
Honey installer

Uso:
  ./installer.sh install      Instala Honey en ~/.local/share/honey
  ./installer.sh uninstall    Elimina la instalacion local de Honey
  ./installer.sh doctor       Revisa dependencias y rutas
  ./installer.sh paths        Muestra rutas usadas por el instalador
  ./installer.sh help         Muestra esta ayuda

Variables:
  HONEY_PREFIX    Prefijo para binarios. Default: ~/.local
  HONEY_HOME      Directorio de instalacion. Default: ~/.local/share/honey

Ejemplos:
  ./installer.sh install
  HONEY_PREFIX="$HOME/.local" ./installer.sh install
EOF
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

print_paths() {
    printf 'SOURCE_DIR=%s\n' "$SOURCE_DIR"
    printf 'PREFIX=%s\n' "$PREFIX"
    printf 'INSTALL_DIR=%s\n' "$INSTALL_DIR"
    printf 'BIN_DIR=%s\n' "$BIN_DIR"
    printf 'BIN_PATH=%s\n' "$BIN_PATH"
}

copy_runtime() {
    mkdir -p "$INSTALL_DIR" "$BIN_DIR"

    cp -f "$SOURCE_DIR/README.md" "$INSTALL_DIR/README.md"
    cp -f "$SOURCE_DIR/installer.sh" "$INSTALL_DIR/installer.sh"
    cp -f "$SOURCE_DIR/CHANGELOG.md" "$INSTALL_DIR/CHANGELOG.md" 2>/dev/null || true
    cp -f "$SOURCE_DIR/LICENSE" "$INSTALL_DIR/LICENSE" 2>/dev/null || true

    rm -rf "$INSTALL_DIR/bin" "$INSTALL_DIR/docs" "$INSTALL_DIR/examples"
    cp -R "$SOURCE_DIR/bin" "$INSTALL_DIR/bin"
    cp -R "$SOURCE_DIR/docs" "$INSTALL_DIR/docs"
    cp -R "$SOURCE_DIR/examples" "$INSTALL_DIR/examples"

    chmod +x "$INSTALL_DIR/bin/honey"
    chmod +x "$INSTALL_DIR/installer.sh"
}

install_runtime() {
    [[ -f "$SOURCE_DIR/bin/honey" ]] || fail "No existe $SOURCE_DIR/bin/honey"

    copy_runtime
    ln -sfn "$INSTALL_DIR/bin/honey" "$BIN_PATH"

    ok "$APP_NAME $APP_VERSION instalado en $INSTALL_DIR"
    ok "Comando disponible en $BIN_PATH"

    if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
        warn "$BIN_DIR no esta en PATH. Agrega: export PATH=\"\$HOME/.local/bin:\$PATH\""
    fi
}

uninstall_runtime() {
    if [[ -L "$BIN_PATH" ]] && [[ "$(readlink "$BIN_PATH")" == "$INSTALL_DIR/bin/honey" ]]; then
        rm -f "$BIN_PATH"
        ok "Shim eliminado: $BIN_PATH"
    elif [[ -e "$BIN_PATH" ]]; then
        warn "No elimine $BIN_PATH porque no parece pertenecer a esta instalacion."
    fi

    if [[ -d "$INSTALL_DIR" ]]; then
        rm -rf "$INSTALL_DIR"
        ok "Instalacion eliminada: $INSTALL_DIR"
    else
        warn "No existe instalacion en $INSTALL_DIR"
    fi
}

doctor() {
    printf '%s %s\n' "$APP_NAME" "$APP_VERSION"
    print_paths
    printf '\nDependencias:\n'

    if command_exists bash; then ok "bash encontrado"; else fail "bash no encontrado"; fi
    if command_exists git; then ok "git encontrado"; else warn "git no encontrado, solo afecta publicacion/desarrollo"; fi

    printf '\nEstado:\n'
    if [[ -x "$INSTALL_DIR/bin/honey" ]]; then ok "runtime instalado"; else warn "runtime no instalado"; fi
    if [[ -L "$BIN_PATH" ]]; then ok "shim honey existe"; else warn "shim honey no existe"; fi
    if [[ ":$PATH:" == *":$BIN_DIR:"* ]]; then ok "$BIN_DIR esta en PATH"; else warn "$BIN_DIR no esta en PATH"; fi
}

main() {
    local command_name="${1:-install}"

    case "$command_name" in
        install) install_runtime ;;
        uninstall) uninstall_runtime ;;
        doctor) doctor ;;
        paths) print_paths ;;
        help|-h|--help) usage ;;
        *)
            usage
            fail "Comando desconocido: $command_name"
            ;;
    esac
}

main "$@"


#!/usr/bin/env bash
# Honey installer
# Permite instalar Honey como reemplazo nativo de bajo nivel (libfreetype.so.6)
# o mediante el metodo tradicional de usuario (Fontconfig, Electron y sesion).

set -Eeuo pipefail

APP_NAME="Honey"
APP_VERSION="0.7.0"
SOURCE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PREFIX="${HONEY_PREFIX:-$HOME/.local}"
INSTALL_DIR="${HONEY_HOME:-$HOME/.local/share/honey}"
BIN_DIR="$PREFIX/bin"
BIN_PATH="$BIN_DIR/honey"

if [[ -d "$HOME/.cargo/bin" && ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

green='\033[0;32m'
yellow='\033[1;33m'
red='\033[0;31m'
cyan='\033[0;36m'
bold='\033[1m'
reset='\033[0m'

ok() { printf '%bOK%b %s\n' "$green" "$reset" "$1"; }
info() { printf '%bINFO%b %s\n' "$cyan" "$reset" "$1"; }
warn() { printf '%bWARN%b %s\n' "$yellow" "$reset" "$1"; }
fail() { printf '%bERROR%b %s\n' "$red" "$reset" "$1" >&2; exit 1; }

usage() {
    cat <<'EOF'
Honey installer

Uso:
  ./installer.sh [install]      Inicia la instalacion (menu interactivo o con opcion)
  ./installer.sh install-native Instala Honey Rust Core como reemplazo nativo de libfreetype.so.6 (sudo)
  ./installer.sh install-user   Instala Honey en modo tradicional de usuario (sin sudo)
  ./installer.sh status-system  Muestra si el sistema ejecuta Honey Rust Core o FreeType
  ./installer.sh restore-system Restaura libfreetype.so.6 oficial de la distribucion (sudo)
  ./installer.sh apply          Aplica la configuracion tradicional de usuario
  ./installer.sh reset          Retira la configuracion tradicional de usuario
  ./installer.sh uninstall      Elimina Honey y restaura configuraciones previas
  ./installer.sh doctor         Revisa dependencias y estado
  ./installer.sh paths          Muestra rutas usadas por el instalador
  ./installer.sh help           Muestra esta ayuda

Opciones para 'install':
  --native, --system            Instalacion de bajo nivel (reemplazo libfreetype.so.6 con Rust)
  --user, --traditional         Instalacion tradicional de usuario (Fontconfig, Electron y sesion)

Variables de entorno:
  HONEY_PREFIX    Prefijo para binarios. Default: ~/.local
  HONEY_HOME      Directorio de instalacion. Default: ~/.local/share/honey
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

copy_tree() {
    local name="$1"
    [[ -d "$SOURCE_DIR/$name" ]] || return 0
    rm -rf "$INSTALL_DIR/$name"
    cp -R "$SOURCE_DIR/$name" "$INSTALL_DIR/$name"
}

install_files() {
    [[ -f "$SOURCE_DIR/bin/honey" ]] || fail "No existe $SOURCE_DIR/bin/honey"
    [[ -f "$SOURCE_DIR/config/fontconfig/99-honey.conf" ]] || fail "No existe el perfil fontconfig de Honey"

    mkdir -p "$INSTALL_DIR" "$BIN_DIR"

    cp -f "$SOURCE_DIR/README.md" "$INSTALL_DIR/README.md"
    cp -f "$SOURCE_DIR/installer.sh" "$INSTALL_DIR/installer.sh"
    cp -f "$SOURCE_DIR/CHANGELOG.md" "$INSTALL_DIR/CHANGELOG.md" 2>/dev/null || true
    cp -f "$SOURCE_DIR/LICENSE" "$INSTALL_DIR/LICENSE" 2>/dev/null || true
    cp -f "$SOURCE_DIR/Cargo.toml" "$INSTALL_DIR/Cargo.toml" 2>/dev/null || true

    copy_tree "bin"
    copy_tree "config"
    copy_tree "docs"
    copy_tree "examples"
    copy_tree "native"
    copy_tree "crates"
    copy_tree "shaders"
    copy_tree "packages"

    if [[ -f "$SOURCE_DIR/target/release/libhoney_shim.so" ]]; then
        mkdir -p "$INSTALL_DIR/target/release"
        cp -f "$SOURCE_DIR/target/release/libhoney_shim.so" "$INSTALL_DIR/target/release/libhoney_shim.so"
        cp -f "$SOURCE_DIR/target/release/libhoney_shim.so" "$INSTALL_DIR/libhoney_shim.so"
    fi

    chmod +x "$INSTALL_DIR/bin/honey"
    chmod +x "$INSTALL_DIR/installer.sh"
    [[ -f "$INSTALL_DIR/examples/check-rendering.sh" ]] && chmod +x "$INSTALL_DIR/examples/check-rendering.sh"
    ln -sfn "$INSTALL_DIR/bin/honey" "$BIN_PATH"
}

run_honey() {
    local action="$1"
    shift || true

    if [[ -x "$INSTALL_DIR/bin/honey" ]]; then
        "$INSTALL_DIR/bin/honey" "$action" "$@"
    elif [[ -x "$SOURCE_DIR/bin/honey" ]]; then
        "$SOURCE_DIR/bin/honey" "$action" "$@"
    else
        fail "No se encontro el comando honey"
    fi
}

ensure_rust_binary_built() {
    local source_shim="$SOURCE_DIR/target/release/libhoney_shim.so"
    if [[ ! -f "$source_shim" ]]; then
        info "Compilando Honey Rust Core en perfil release como usuario..."
        local cargo_cmd="cargo"
        if ! command -v cargo >/dev/null 2>&1 && [[ -x "$HOME/.cargo/bin/cargo" ]]; then
            cargo_cmd="$HOME/.cargo/bin/cargo"
        fi
        (cd "$SOURCE_DIR" && "$cargo_cmd" build --release -p honey-shim)
    fi
    [[ -f "$source_shim" ]] || fail "No se pudo compilar $source_shim"
    ok "Binario compilado verificado: $source_shim"
}

install_native_mode() {
    info "Preparando instalacion en Modo Nativo de Bajo Nivel (Rust / Quartz Parity)..."

    # 1. Asegurar compilacion en espacio de usuario con cargo antes de pedir sudo
    ensure_rust_binary_built

    # 2. Instalar y copiar archivos (incluye target/release/libhoney_shim.so)
    install_files

    # 3. Invocar sudo unicamente para registrar el reemplazo de biblioteca
    if [[ $EUID -eq 0 ]]; then
        run_honey install-system
    else
        info "Se requieren privilegios de superusuario para registrar el reemplazo en el sistema."
        sudo "$INSTALL_DIR/bin/honey" install-system
    fi

    ok "Instalacion de bajo nivel completada con exito."
    ok "Todas las aplicaciones del sistema ahora utilizan Honey Rust Core."
}

install_user_mode() {
    info "Preparando instalacion en Modo Tradicional de Usuario (Fontconfig / Electron)..."
    install_files

    ok "$APP_NAME $APP_VERSION instalado en $INSTALL_DIR"
    ok "Comando disponible en $BIN_PATH"

    if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
        warn "$BIN_DIR no esta en PATH. Agrega: export PATH=\"$BIN_DIR:\$PATH\""
    fi

    info "Aplicando perfil tipografico en espacio de usuario..."
    run_honey apply

    ok "Modo tradicional configurado exitosamente."
}

prompt_install_mode() {
    printf '%b=================================================================%b\n' "$bold" "$reset"
    printf '%b  Instalador de Honey Tipografia macOS Paridad Quartz v%s        %b\n' "$bold" "$APP_VERSION" "$reset"
    printf '%b=================================================================%b\n\n' "$bold" "$reset"
    printf 'Seleccione el metodo de instalacion deseado:\n\n'
    printf '  %b1) Reemplazo Nativo de Bajo Nivel (Recomendado)%b\n' "$bold" "$reset"
    printf '     - Sustituye libfreetype.so.6 con el motor Rust de Honey (cdylib).\n'
    printf '     - Paridad analitica Quartz, dilatacion optica y gamma lineal pura.\n'
    printf '     - Aplica a TODO el sistema operativo (GTK, Qt, Firefox, Electron).\n'
    printf '     - Cero parches de configuracion; requiere privilegios sudo.\n\n'
    printf '  %b2) Modo Tradicional de Usuario (Fontconfig y Parcheado)%b\n' "$bold" "$reset"
    printf '     - Configura ~/.config/fontconfig/conf.d/99-honey.conf.\n'
    printf '     - Sincroniza variables FREETYPE_PROPERTIES en sesion y flags de Electron.\n'
    printf '     - Inyecta hojas de estilo en navegadores (Firefox userChrome.css).\n'
    printf '     - No requiere privilegios de superusuario (sin sudo).\n\n'

    local choice=""
    if [[ -t 0 ]]; then
        read -r -p "Ingrese su opcion [1 o 2] (Default: 1): " choice
        choice="${choice:-1}"
    else
        choice="1"
    fi

    case "$choice" in
        1) install_native_mode ;;
        2) install_user_mode ;;
        *)
            warn "Opcion invalida: $choice. Usando opcion por defecto (1: Nativo)."
            install_native_mode
            ;;
    esac
}

uninstall_honey() {
    if [[ -x "$INSTALL_DIR/bin/honey" ]]; then
        "$INSTALL_DIR/bin/honey" reset || true
    fi

    local conf_file="/etc/ld.so.conf.d/00-honey.conf"
    local deb_divert="/usr/lib/x86_64-linux-gnu/libfreetype.so.6.distrib"
    if [[ -f "$conf_file" || -e "$deb_divert" ]]; then
        info "Detectado reemplazo nativo en sistema. Restaurando..."
        if [[ $EUID -eq 0 ]]; then
            run_honey restore-system || true
        else
            sudo "$INSTALL_DIR/bin/honey" restore-system || true
        fi
    fi

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
    if command_exists cargo; then ok "cargo encontrado"; else warn "cargo no encontrado (requerido para compilar Rust core)"; fi
    if command_exists gcc; then ok "gcc encontrado"; else warn "gcc no encontrado"; fi
    if command_exists fc-cache; then ok "fc-cache encontrado"; else warn "fc-cache no encontrado"; fi
    if command_exists fc-match; then ok "fc-match encontrado"; else warn "fc-match no encontrado"; fi
    if command_exists git; then ok "git encontrado"; else warn "git no encontrado, solo afecta publicacion/desarrollo"; fi

    printf '\nEstado de componentes:\n'
    if [[ -x "$INSTALL_DIR/bin/honey" ]]; then ok "Honey CLI instalado en $INSTALL_DIR"; else warn "Honey CLI no instalado"; fi
    if [[ -L "$BIN_PATH" ]]; then ok "Symlink honey existe en $BIN_PATH"; else warn "Symlink honey no existe"; fi
    if [[ ":$PATH:" == *":$BIN_DIR:"* ]]; then ok "$BIN_DIR esta en PATH"; else warn "$BIN_DIR no esta en PATH"; fi

    printf '\nEstado de motor nativo:\n'
    run_honey status-system || true
}

main() {
    local command_name="${1:-install}"
    shift || true

    case "$command_name" in
        install)
            local mode=""
            while [[ $# -gt 0 ]]; do
                case "$1" in
                    --native|--system) mode="native" ;;
                    --user|--traditional) mode="user" ;;
                    *) warn "Opcion desconocida para install: $1" ;;
                esac
                shift || true
            done

            if [[ "$mode" == "native" ]]; then
                install_native_mode
            elif [[ "$mode" == "user" ]]; then
                install_user_mode
            else
                prompt_install_mode
            fi
            ;;
        install-native|install-system) install_native_mode ;;
        install-user|install-traditional) install_user_mode ;;
        status-system) run_honey status-system "$@" ;;
        restore-system)
            if [[ $EUID -eq 0 ]]; then
                run_honey restore-system "$@"
            else
                sudo "$INSTALL_DIR/bin/honey" restore-system "$@"
            fi
            ;;
        apply) install_files && run_honey apply ;;
        reset) run_honey reset ;;
        uninstall) uninstall_honey ;;
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

# Guia de Instalacion y Despliegue de Honey

Este documento describe los metodos oficiales de instalacion, compilacion y despliegue del motor tipografico **Honey** en distribuciones GNU/Linux.

---

## 1. Requisitos Previos del Sistema

### Dependencias de Compilacion:
- **Rust y Cargo:** Version 1.75 o superior (`rustc --version`).
- **Compilador C:** GCC o Clang con soporte para C99/C11.
- **Cabeceras de FreeType 2:** Para compilar la suite de verificacion ABI (`freetype-devel` en Fedora, `libfreetype6-dev` en Ubuntu/Debian, `freetype2` en Arch).

---

## 2. Metodos de Instalacion

Honey soporta dos modos de operacion segun el nivel de integracion deseado:

### Metodo 1: Reemplazo Nativo de Bajo Nivel a Nivel de Sistema (Recomendado)
Sustituye binariamente la biblioteca compartida `libfreetype.so.6` en el sistema. Todas las aplicaciones graficas (GTK, Qt, Firefox, Chrome, Electron, LibreOffice) adoptan automaticamente el motor Honey con paridad Quartz.

```bash
# 1. Clonar el repositorio
git clone https://github.com/isgaar/Honey.git
cd Honey

# 2. Compilar el workspace en perfil optimizado de produccion
cargo build --release --workspace

# 3. Instalar en el sistema operativo (requiere privilegios sudo)
sudo ./bin/honey install-system
```

#### Que hace `install-system`:
1. Copia `libhoney_shim.so` hacia `/usr/local/lib64/` (o `/usr/local/lib/`).
2. Crea el enlace simbolico `/usr/local/lib64/libfreetype.so.6`.
3. Registra `/usr/local/lib64` en `/etc/ld.so.conf.d/00-honey.conf`.
4. **Seguridad SELinux automatica:** Si el sistema ejecuta SELinux (Fedora/RHEL), invoca automaticamente `restorecon -F` y `chcon -t lib_t` para evitar bloqueos en GDM o SDDM.
5. **Prueba de seguridad en vivo:** Ejecuta inmediatamente una prueba de enlace y renderizado con `fc-match` y `fc-list`. Si detecta cualquier anomalia, revierte de inmediato los cambios protegiendo el entorno grafico.
6. Actualiza la cache de bibliotecas del kernel ejecutando `ldconfig`.

### Metodo 2: Modo Aislado por Aplicacion (`LD_PRELOAD`)
Permite probar Honey en aplicaciones individuales sin alterar las bibliotecas del sistema:

```bash
# Ejecutar Zen Browser con Honey:
LD_PRELOAD=$(pwd)/target/release/libhoney_shim.so zen-browser &

# Ejecutar Firefox con Honey:
LD_PRELOAD=$(pwd)/target/release/libhoney_shim.so MOZ_ENABLE_WAYLAND=1 firefox &

# Ejecutar GNOME Archivos (Nautilus) con Honey:
LD_PRELOAD=$(pwd)/target/release/libhoney_shim.so nautilus --new-window &
```

---

## 3. Empaquetado Nativo para Distribuciones

El directorio [packages/](file:///Users/ismael/Documents/Honey/packages) contiene los manifiestos de empaquetado formal:

### 3.1 Fedora / RHEL (RPM)
Manifiesto disponible en [packages/fedora/honey-freetype.spec](file:///Users/ismael/Documents/Honey/packages/fedora/honey-freetype.spec).
```bash
rpmbuild -ba packages/fedora/honey-freetype.spec
```

### 3.2 Arch Linux (PKGBUILD)
Manifiesto disponible en [packages/arch/PKGBUILD](file:///Users/ismael/Documents/Honey/packages/arch/PKGBUILD).
```bash
cd packages/arch && makepkg -si
```

### 3.3 Ubuntu / Debian (`dpkg-divert`)
En Debian/Ubuntu, `install_system` utiliza `dpkg-divert` para desviar limpiamente el archivo oficial sin que las actualizaciones de APT sobreescriban a Honey:
```bash
sudo dpkg-divert --package honey-freetype --add --rename --divert /usr/lib/x86_64-linux-gnu/libfreetype.so.6.distrib /usr/lib/x86_64-linux-gnu/libfreetype.so.6
```

---

## 4. Reversion del Sistema (Desinstalacion Limpia)

En cualquier momento, el sistema puede restaurarse al motor FreeType original de la distribucion con un unico comando:
```bash
sudo ./bin/honey restore-system
```
Este comando retira los enlaces simbolicos de `/usr/local/lib64`, elimina `/etc/ld.so.conf.d/00-honey.conf`, revierte los desvios de `dpkg-divert` (si aplican) y regenera la cache del cargador con `ldconfig`.

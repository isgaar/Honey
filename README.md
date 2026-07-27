# Honey

Honey es una capa de ajuste tipográfico para Linux. Su objetivo es dejar el
texto más limpio, consistente y suave sin depender de cambios globales del
sistema ni de configuraciones difíciles de recordar.

Honey trabaja en modo usuario: instala un comando `honey`, guarda una plantilla
de `fontconfig` y permite activar o retirar el perfil cuando quieras.

## Qué hace

- Activa antialiasing.
- Usa hinting ligero (`hintslight`) para preservar la geometría y curvas redondas perfectas sin deformar letras.
- Activa `stem darkening` en FreeType (`no-stem-darkening=0`) para dar el cuerpo sólido y redondeo limpio característico de macOS sobre temas oscuros.
- Establece peso Medium 500 para la interfaz manteniendo legibilidad y presencia sin forzar negritas globales.
- Aplica suavizado `antialiased` limpio en la web (`userContent.css`) sin trazos artificiales destructivos (`-webkit-text-stroke`).
- Prefiere render grayscale (`rgba=none`) en lugar de subpixel RGB.
- Desactiva bitmaps incrustados para fuentes escalables.
- Define una prioridad razonable de fuentes sans, serif, mono y emoji.
- Regenera la caché de fuentes cuando `fc-cache` está disponible.
- Agrega una base para documentar y ampliar instrucciones orientadas a apps GTK y KDE.
- Sincroniza GTK, KDE, DBus y systemd user para que las apps lanzadas por KDE hereden Honey.
- Adapta el lanzador KDE de VSCodium cuando una instalación local omite el entorno de sesión.
- Mantiene todo bajo `$HOME`, sin `sudo`.

## Qué incluye

- `installer.sh`: instala o desinstala Honey en modo usuario.
- `bin/honey`: comando principal para aplicar, retirar y revisar el perfil.
- `config/fontconfig/99-honey.conf`: perfil tipográfico versionado.
- `examples/check-rendering.sh`: ejemplo de diagnóstico local.
- `docs/`: documentación extensa de instalación, uso, render, desarrollo y GitHub.
- `scripts/publish-github.sh`: flujo de publicación del repositorio.

## Instalación rápida

Desde esta carpeta:

```bash
chmod +x installer.sh bin/honey examples/check-rendering.sh scripts/publish-github.sh
./installer.sh install
```

Si `~/.local/bin` no está en tu `PATH`, agrega esto a tu shell:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Después aplica el perfil:

```bash
honey apply
honey status
```

Para retirar el perfil:

```bash
honey reset
```

## Uso básico

Ver ayuda:

```bash
honey help
```

Aplicar Honey:

```bash
honey apply
```

Revisar estado:

```bash
honey status
```

Ver rutas usadas:

```bash
honey paths
```

Retirar el perfil:

```bash
honey reset
```

## Rutas importantes

Honey instala sus archivos en:

```bash
$HOME/.local/share/honey
```

El comando queda en:

```bash
$HOME/.local/bin/honey
```

El perfil activo de fontconfig queda en:

```bash
$HOME/.config/fontconfig/conf.d/99-honey.conf
```

Honey también puede ajustar estas rutas de compatibilidad:

```bash
$HOME/.config/gtk-3.0/settings.ini
$HOME/.config/gtk-4.0/settings.ini
$HOME/.config/xsettingsd/xsettingsd.conf
$HOME/.config/kdeglobals
$HOME/.config/codium-flags.conf
$HOME/.mozilla/firefox/<perfil>/chrome/userChrome.css
$HOME/.mozilla/firefox/<perfil>/chrome/userContent.css
$HOME/.mozilla/firefox/<perfil>/user.js
$HOME/.config/environment.d/90-honey.conf
$HOME/.config/plasma-workspace/env/90-honey.sh
```

## GTK y KDE

Estos cambios también buscan sumar más instrucciones para que las aplicaciones
GTK y KDE respeten mejor el perfil tipográfico de Honey. La idea es mantener
una base común desde `fontconfig` y documentar ajustes específicos cuando una
app o toolkit necesite configuración adicional.

En KDE, Honey alinea `XftAntialias`, `XftHintStyle` y `XftSubPixel` dentro de
`kdeglobals`. En GTK, alinea `gtk-xft-antialias`, `gtk-xft-hinting`,
`gtk-xft-hintstyle` y `gtk-xft-rgba`. En Firefox, agrega una hoja
`userChrome.css` limitada a la interfaz y `userContent.css` con un ajuste de
trazo neutral; no impone fuentes, pesos, tamaños ni reglas de diseño.

## VSCodium, VS Code, Cursor y Aplicaciones Electron

Las aplicaciones Electron (VS Code, VSCodium, Cursor, Antigravity, Obsidian, Slack, Discord, Element, Spotify, etc.) usan el motor Chromium/Skia. Por defecto en Linux, este motor puede renderizar fuentes delgadas o con franjas subpixel RGB.

Honey configura las aplicaciones Electron para imitar la calidad y suavizado de **macOS**:
- Configura archivos de banderas en `~/.config/` (`code-flags.conf`, `codium-flags.conf`, `electron-flags.conf`, etc.) con `--disable-lcd-text`, `--font-render-hinting=slight` y `--enable-font-antialiasing`.
- Habilita `truetype:no-stem-darkening=0` en FreeType para dar volumen y solidez a fuentes TrueType (Inter, JetBrains Mono, SF Pro).
- Inyecta la capa de sesión mediante `environment.d`, `plasma-workspace/env`, DBus y systemd user.

Después de `honey apply`, las aplicaciones Electron heredan:

```text
HONEY_ACTIVE=1
HONEY_FONTCONFIG=$HOME/.config/fontconfig/conf.d/99-honey.conf
FREETYPE_PROPERTIES=truetype:interpreter-version=40 truetype:no-stem-darkening=0 cff:no-stem-darkening=0 autofitter:no-stem-darkening=0 type1:no-stem-darkening=0 t1mode:no-stem-darkening=0
```

Para lanzar cualquier aplicación Electron directamente con el perfil macOS de Honey:

```bash
honey launch-electron cursor
honey launch-code .
honey launch-codium .
honey launch-honey-codium .
```

## Documentación

- [Instalación](docs/INSTALL.md)
- [Uso](docs/USAGE.md)
- [Modelo de render](docs/RENDERING.md)
- [Desarrollo](docs/DEVELOPMENT.md)
- [Publicación en GitHub](docs/GITHUB.md)
- [Solución de problemas](docs/TROUBLESHOOTING.md)

## Estado del proyecto

Honey está en etapa inicial. La prioridad de esta base es que el proyecto tenga:

- instalación reproducible,
- cambios fáciles de revisar,
- activación y retiro sin fricción,
- documentación suficiente para ajustar el render con confianza.

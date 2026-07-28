# Changelog

## 0.6.0

- Agrega soporte integral para aplicaciones Electron (VS Code, VSCodium, Cursor, Antigravity, Obsidian, Slack, Discord, Element, Spotify, etc.) para imitar la calidad de renderizado de texto de macOS.
- Elimina los bordes delgados y afilados cambiando el hinting a `hintnone` (`hinting=false` / `--font-render-hinting=none`) en Fontconfig, GTK 3/4, xsettingsd, KDE y Electron.
- Promueve el peso base de la interfaz y fuentes predeterminadas a `SemiBold` (peso 600) en Fontconfig y KDE para dar volumen, solidez y cuerpo grueso a ventanas, menús y editores.
- Habilita `truetype:no-stem-darkening=0` en `FREETYPE_PROPERTIES` y `<edit name="stemdarkening">` en Fontconfig para asegurar la solidez y grosor de trazo en fuentes TrueType (Inter, JetBrains Mono, SF Pro, Roboto).
- Sincroniza banderas globales e individuales de Electron (`--disable-lcd-text`, `--font-render-hinting=none`, `--enable-font-antialiasing`) a través de archivos `.conf` en `~/.config/`.
- Añade el comando `honey launch-electron COMANDO [ARGS]` (y alias como `launch-code` y `launch-cursor`) para ejecutar cualquier aplicación Electron con el perfil de render macOS.

## 0.5.3

- Activa FreeType stem darkening controlado (`no-stem-darkening=0`) y peso base Medium 500 para dar a la tipografía el cuerpo justo, solidez y redondeo limpio característico del renderizado de macOS.
- Mantiene hinting ligero (`hintslight`) para preservar la curvatura geométrica perfecta en editores, terminales y ventanas.

## 0.5.2

- Implementa engrosamiento tipográfico inteligente con hinting ligero (`hintslight`) y no-stem-darkening en FreeType.
- Corrije la renderización en sitios web eliminando el destructivo `-webkit-text-stroke` de `userContent.css`.
- Restaura el antialiased smoothing nativo respetando los pesos tipográficos naturales en Firefox, GTK, KDE y Electron.
- Añade renderizado y tipografía nativa estilo macOS (SF Pro/Inter, letter-spacing -0.15px, peso 500 en pestaña activa) para la interfaz de Firefox en `userChrome.css`.

## 0.5.1

- Añade un trazo web neutral para todas las páginas sin modificar su diseño.
- No fuerza familia, peso, tamaño, color, márgenes ni estilos propios del sitio.

## 0.5.0

- Mueve el ajuste de Firefox a `userChrome.css`, limitado a su interfaz, con peso semibold 600.
- Retira por completo el override de páginas web y la regla específica de Gemini.
- Conserva iconos y SVG sin alterar.

## 0.4.9

- Refuerza el texto anidado de `gemini.google.com` con peso 500 y trazo 0.16 px.
- Mantiene SVG, iconos y la interfaz de Firefox fuera de la regla.

## 0.4.8

- Retira el peso Inter Medium de la interfaz GTK para no engrosar el navegador.
- Añade `userContent.css` de Firefox con peso 500 y un trazo sutil solo para páginas web.
- Habilita la hoja por perfil y restaura los archivos originales con `honey reset`.

## 0.4.7

- Eleva a 500 el peso base de las fuentes KDE para dar más cuerpo a Dolphin y sus diálogos.
- Conserva pesos KDE semibold/bold existentes y mantiene GTK/Firefox separado.

## 0.4.6

- Usa Inter Medium únicamente en la interfaz GTK/Firefox para un trazo geométrico más visible.
- Mantiene la fuente KDE y el contenido web en sus pesos normales.
- Conserva hinting medio y antialiasing en escala de grises.

## 0.4.5

- Usa hinting medio como ajuste sutil de grosor sin forzar Inter Medium ni pesos KDE globales.
- Mantiene la escala de grises y los pesos explícitos de cada aplicación.

## 0.4.4

- Retira el peso global Inter Medium porque endurecía demasiado la interfaz de Firefox y GTK.
- Vuelve a `hintslight` como valor predeterminado, conservando `rgba=none` y la capa de sesión KDE.

## 0.4.3

- Ajuste experimental de peso de interfaz, reemplazado por el perfil conservador en 0.4.4.

## 0.4.2

- Usa hinting medio para dar más cuerpo al texto manteniendo antialiasing en escala de grises.
- Sincroniza el mismo hintstyle en Fontconfig, GTK, xsettingsd, KDE y lanzadores Electron.

## 0.4.1

- Corrige lanzadores locales de VSCodium que eludian el entorno de sesión de Honey.
- Hace que `launch-codium` y `launch-honey-codium` exporten explícitamente el perfil de render.
- Prefiere el VSCodium visible en `PATH`, conservando compatibilidad con instalaciones en `/opt`.
- Genera un override KDE reversible y conserva el lanzador original antes de modificarlo.

## 0.4.0

- Cambia la estrategia principal a una capa de sesión KDE mediante `environment.d` y `plasma-workspace/env`.
- Importa `HONEY_ACTIVE`, `HONEY_FONTCONFIG` y `FREETYPE_PROPERTIES` en systemd user/DBus cuando es posible.
- Deja de depender de lanzadores `.desktop` por aplicación como mecanismo principal.
- Restaura flags y lanzadores locales heredados de pruebas anteriores cuando `honey apply` o `honey reset` encuentran respaldos.

## 0.3.2

- Agrega `codium-honey.desktop` como instancia separada para KDE.
- Agrega `honey launch-honey-codium` con `user-data-dir` y extensiones aisladas.
- Expone la ruta del launcher Honey en `honey status`, `doctor` y `paths`.

## 0.3.1

- Corrige el lanzador de Codium para usar `/usr/bin/codium`, que si reenvia flags a Electron.
- Corrige `honey launch-codium` para usar el wrapper en lugar del binario directo.

## 0.3.0

- Agrega compatibilidad para GTK 3, GTK 4, xsettingsd y KDE/kdeglobals.
- Alinea flags de Electron/Codium para que VSCodium use hinting ligero.
- Agrega override de lanzador `.desktop` para que VSCodium abierto desde KDE use Honey.
- Agrega `honey launch-codium` para lanzar Codium con flags explicitamente.
- Guarda originales de configs modificadas para poder restaurar con `honey reset`.
- Amplia `honey status` y `honey doctor` con diagnostico de toolkits.

## 0.2.0

- Reorienta Honey como capa tipografica de usuario.
- Agrega perfil `fontconfig` versionado.
- Cambia la CLI a `apply`, `reset`, `status`, `doctor` y `paths`.
- Actualiza documentacion para instalacion, uso y modelo de render.
- Agrega ejemplo de diagnostico con `fc-match`.

## 0.1.0

- Estructura inicial del proyecto Honey.
- Instalador local con `install`, `uninstall`, `doctor` y `paths`.
- CLI inicial.
- Documentacion inicial.

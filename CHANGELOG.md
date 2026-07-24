# Changelog

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

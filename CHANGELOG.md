# Changelog

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

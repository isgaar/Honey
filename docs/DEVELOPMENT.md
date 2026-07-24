# Desarrollo de Honey

Esta guia esta pensada para modificar Honey sin perder orden.

## Estructura del proyecto

```text
Honey/
  README.md
  installer.sh
  bin/
    honey
  config/
    fontconfig/
      99-honey.conf
    firefox/
      userChrome.css
  docs/
    INSTALL.md
    USAGE.md
    RENDERING.md
    DEVELOPMENT.md
    GITHUB.md
    TROUBLESHOOTING.md
  examples/
    check-rendering.sh
  scripts/
    publish-github.sh
```

## Archivos principales

`installer.sh`

Instala el comando y la estructura de Honey bajo `$HOME/.local/share/honey`.
Debe mantenerse conservador: no usar `sudo`, no borrar archivos ajenos y no
activar cambios sin un comando explicito.

`bin/honey`

CLI principal. Aplica, retira y diagnostica el perfil tipografico, incluyendo
compatibilidad GTK/KDE/Electron y entorno de sesión KDE/DBus.

`config/fontconfig/99-honey.conf`

Perfil real que se copia a `~/.config/fontconfig/conf.d/99-honey.conf` cuando se
ejecuta `honey apply`.

`config/firefox/userChrome.css`

Hoja cargada por los perfiles Firefox detectados. Solo afecta la interfaz;
`bin/honey` la respalda y la instala en `chrome/userChrome.css` junto con la
preferencia que habilita personalizaciones del perfil.

`docs/RENDERING.md`

Explica las decisiones del perfil. Si cambias `99-honey.conf`, actualiza este
archivo.

## Flujo recomendado

1. Cambia el codigo o el perfil.
2. Ejecuta diagnostico desde el repo:

```bash
./bin/honey doctor
```

3. Instala en modo usuario:

```bash
./installer.sh install
```

4. Aplica el perfil:

```bash
honey apply
```

5. Revisa estado:

```bash
honey status
```

6. Si algo no convence, retira el perfil y restaura configs gestionadas:

```bash
honey reset
```

## Estilo de Bash

Preferencias del proyecto:

- usar `set -Eeuo pipefail`;
- envolver errores con mensajes claros;
- evitar `sudo`;
- mantener funciones pequenas;
- usar nombres descriptivos;
- preferir rutas bajo `$HOME`;
- no depender de herramientas raras para la instalacion base.

## Versionado

La version vive por ahora en:

- `installer.sh` como `APP_VERSION`;
- `bin/honey` como `VERSION`;
- `CHANGELOG.md`.

Cuando cambies version, actualiza los tres lugares.

## Pruebas manuales

Diagnostico:

```bash
./bin/honey doctor
```

Instalacion:

```bash
./installer.sh install
honey version
```

Aplicacion:

```bash
honey apply
honey status
```

Retiro:

```bash
honey reset
```

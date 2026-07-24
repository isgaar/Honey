# Uso de Honey

Honey se usa con el comando `honey`.

## Ver ayuda

```bash
honey help
```

## Ver version

```bash
honey version
```

## Aplicar el perfil

```bash
  honey apply
```

Este comando:

- crea `~/.config/fontconfig/conf.d` si hace falta;
- respalda un `99-honey.conf` previo si no fue creado por Honey;
- copia el perfil de Honey;
- alinea GTK 3, GTK 4, xsettingsd y KDE;
- instala una capa de sesión para KDE con `environment.d` y `plasma-workspace/env`;
- importa el entorno Honey a systemd user y DBus cuando la sesión lo permite;
- adapta con respaldo el lanzador KDE de VSCodium cuando existe;
- habilita `userChrome.css` para la interfaz y un trazo web neutral sin alterar diseños;
- regenera cache con `fc-cache` cuando esta disponible.

## Sesión KDE

En KDE, la ruta estable es que las apps lanzadas desde el menú hereden
variables de sesión:

```text
HONEY_ACTIVE=1
HONEY_FONTCONFIG=~/.config/fontconfig/conf.d/99-honey.conf
FREETYPE_PROPERTIES=truetype:interpreter-version=40
```

Honey escribe esas variables en:

```bash
~/.config/environment.d/90-honey.conf
~/.config/plasma-workspace/env/90-honey.sh
```

Algunas instalaciones locales de VSCodium usan un lanzador que no conserva ese
entorno. Honey crea un override KDE gestionado y guarda el original antes de
redirigirlo por `honey launch-codium`.

## Diagnóstico con VSCodium

```bash
honey launch-honey-codium .
```

Ese comando abre una instancia aislada para comparar render y confirmar que
Honey funciona sin depender de una ventana vieja.

```bash
~/.config/VSCodium-Honey
~/.local/share/honey/codium/extensions
```

## Ver estado

```bash
honey status
```

Muestra:

- si el perfil esta activo;
- donde esta el archivo fuente;
- donde esta el archivo aplicado;
- que fuentes resuelve `fontconfig` para familias comunes;
- que hinting estan usando GTK, KDE y Codium;
- que perfiles de Firefox tienen activos los renders de interfaz y web neutral.
- si `environment.d` y el hook de Plasma estan activos.

## Retirar el perfil

```bash
honey reset
```

Honey solo elimina el archivo activo cuando detecta su marca interna:

```text
Managed by Honey
```

Si el archivo no parece suyo, Honey no lo borra. Aun asi intenta restaurar las
configuraciones GTK/KDE/Electron que si tenga registradas.

Tambien restaura las configuraciones originales que Honey guardo en:

```bash
~/.local/state/honey/originals
```

Esto incluye `userChrome.css`, `userContent.css` y `user.js` de los perfiles
Firefox detectados.

## Diagnostico

```bash
honey doctor
```

Sirve para revisar:

- Bash;
- `fc-cache`;
- `fc-match`;
- perfil fuente;
- configuraciones GTK/KDE/Codium;
- rutas de configuracion.

## Rutas

```bash
honey paths
```

Salida esperada:

```text
ROOT_DIR=/home/ismael/.local/share/honey
PROFILE_SOURCE=/home/ismael/.local/share/honey/config/fontconfig/99-honey.conf
PROFILE_TARGET=/home/ismael/.config/fontconfig/conf.d/99-honey.conf
STATE_DIR=/home/ismael/.local/state/honey
BACKUP_DIR=/home/ismael/.local/state/honey/backups
ORIGINAL_DIR=/home/ismael/.local/state/honey/originals
```

## Flujo recomendado

```bash
./installer.sh install
honey apply
honey status
```

Despues de aplicar, reabre las aplicaciones donde quieras ver el cambio:
terminal, editor, navegador, paneles y lanzadores.

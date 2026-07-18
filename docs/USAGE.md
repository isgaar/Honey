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
- alinea flags de Electron/Codium para VSCodium;
- crea un override local de `codium.desktop` para KDE;
- regenera cache con `fc-cache` cuando esta disponible.

## Lanzar VSCodium con Honey

Si quieres comprobar el render sin usar el lanzador del menu:

```bash
honey launch-codium .
```

Eso abre Codium pasando los flags de Honey directamente al proceso.

## Ver estado

```bash
honey status
```

Muestra:

- si el perfil esta activo;
- donde esta el archivo fuente;
- donde esta el archivo aplicado;
- que fuentes resuelve `fontconfig` para familias comunes;
- que hinting estan usando GTK, KDE y Codium.
- si el lanzador local de Codium ya incluye los flags de Honey.

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

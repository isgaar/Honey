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
- regenera cache con `fc-cache` cuando esta disponible.

## Ver estado

```bash
honey status
```

Muestra:

- si el perfil esta activo;
- donde esta el archivo fuente;
- donde esta el archivo aplicado;
- que fuentes resuelve `fontconfig` para familias comunes.

## Retirar el perfil

```bash
honey reset
```

Honey solo elimina el archivo activo cuando detecta su marca interna:

```text
Managed by Honey
```

Si el archivo no parece suyo, termina con error para evitar borrar trabajo
manual.

## Diagnostico

```bash
honey doctor
```

Sirve para revisar:

- Bash;
- `fc-cache`;
- `fc-match`;
- perfil fuente;
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
```

## Flujo recomendado

```bash
./installer.sh install
honey apply
honey status
```

Despues de aplicar, reabre las aplicaciones donde quieras ver el cambio:
terminal, editor, navegador, paneles y lanzadores.


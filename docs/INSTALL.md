# Instalacion de Honey

Esta guia explica como instalar Honey desde la carpeta del proyecto.

## Requisitos

Honey necesita:

- Bash 4 o superior.
- Permisos de escritura en tu usuario.
- `git` solo si vas a publicar o trabajar con versiones.

No necesita `sudo` para instalarse.

## Instalacion recomendada

Desde `Documentos/Honey`:

```bash
chmod +x installer.sh bin/honey scripts/publish-github.sh
./installer.sh install
```

El instalador copia los archivos a:

```bash
$HOME/.local/share/honey
```

Y crea un enlace ejecutable en:

```bash
$HOME/.local/bin/honey
```

## PATH

Para poder escribir `honey` desde cualquier carpeta, `~/.local/bin` debe estar
en tu `PATH`.

Compruebalo:

```bash
printf '%s\n' "$PATH"
```

Si falta, agrega esto a `~/.bashrc`, `~/.zshrc` o el archivo equivalente:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Recarga la terminal:

```bash
source ~/.bashrc
```

## Verificar instalacion

```bash
honey version
honey doctor
honey run examples/hello.hny
```

Si estas fuera de la carpeta del proyecto, ejecuta el ejemplo instalado:

```bash
honey run "$HOME/.local/share/honey/examples/hello.hny"
```

## Instalar en otra ruta

Puedes cambiar el prefijo de binarios:

```bash
HONEY_PREFIX="$HOME/.local" ./installer.sh install
```

Puedes cambiar el directorio completo de instalacion:

```bash
HONEY_HOME="$HOME/apps/honey" ./installer.sh install
```

Tambien puedes usar ambas variables:

```bash
HONEY_PREFIX="$HOME/.local" HONEY_HOME="$HOME/apps/honey" ./installer.sh install
```

## Desinstalar

```bash
./installer.sh uninstall
```

Esto elimina:

- el enlace `~/.local/bin/honey`, si apunta a la instalacion de Honey;
- el directorio `~/.local/share/honey`.

No elimina tus programas `.hny` creados en otras carpetas.

## Diagnostico

```bash
./installer.sh doctor
```

Ese comando muestra:

- rutas usadas por el instalador,
- si `bash` esta disponible,
- si `git` esta disponible,
- si Honey ya esta instalado,
- si `~/.local/bin` esta en `PATH`.


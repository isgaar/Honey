# Instalacion de Honey

Esta guia explica como instalar Honey sin tocar rutas del sistema.

## Requisitos

Honey necesita:

- Bash.
- `fontconfig`, normalmente ya instalado en escritorios Linux.
- `fc-cache` para regenerar cache de fuentes.
- `fc-match` para diagnostico.
- `git` solo si vas a publicar o trabajar con versiones.

No necesita `sudo`.

## Instalacion recomendada

Desde `Documentos/Honey`:

```bash
chmod +x installer.sh bin/honey examples/check-rendering.sh scripts/publish-github.sh
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

Si falta, agrega esto a tu shell:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Recarga la terminal:

```bash
source ~/.bashrc
```

## Activar el perfil

Instalar Honey solo instala el comando y sus archivos. Para activar el perfil:

```bash
honey apply
```

Honey escribira:

```bash
$HOME/.config/fontconfig/conf.d/99-honey.conf
```

Tambien sincronizara, con respaldos, configuraciones de GTK, KDE, xsettingsd y
Electron/Codium cuando existan. En KDE instala una capa de sesión para que el
menú y otros lanzadores hereden Honey. Si detecta VSCodium, adapta de forma
reversible su lanzador KDE para cubrir instalaciones locales que omiten el
entorno de sesión.

Despues regenera la cache si `fc-cache` esta disponible.

## Verificar instalacion

```bash
honey version
honey doctor
honey status
```

Tambien puedes ejecutar el ejemplo:

```bash
~/.local/share/honey/examples/check-rendering.sh
```

Para abrir una instancia aislada de diagnóstico:

```bash
honey launch-honey-codium .
```

## Retirar el perfil

```bash
honey reset
```

Eso elimina `99-honey.conf` solo si el archivo contiene la marca de Honey y
restaura los originales guardados de GTK/KDE/Electron.

## Desinstalar todo

```bash
./installer.sh uninstall
```

Esto elimina:

- el perfil activo de Honey, si existe;
- el enlace `~/.local/bin/honey`, si apunta a esta instalacion;
- el directorio `~/.local/share/honey`.

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

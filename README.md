# Honey

Honey es una capa de ajuste tipografico para Linux. Su objetivo es dejar el
texto mas limpio, consistente y suave sin depender de cambios globales del
sistema ni de configuraciones dificiles de recordar.

Honey trabaja en modo usuario: instala un comando `honey`, guarda una plantilla
de `fontconfig` y permite activar o retirar el perfil cuando quieras.

## Que hace

- Activa antialiasing.
- Usa hinting ligero para evitar trazos duros.
- Prefiere render grayscale en lugar de subpixel RGB.
- Desactiva bitmaps incrustados para fuentes escalables.
- Define una prioridad razonable de fuentes sans, serif, mono y emoji.
- Regenera la cache de fuentes cuando `fc-cache` esta disponible.
- Mantiene todo bajo `$HOME`, sin `sudo`.

## Que incluye

- `installer.sh`: instala o desinstala Honey en modo usuario.
- `bin/honey`: comando principal para aplicar, retirar y revisar el perfil.
- `config/fontconfig/99-honey.conf`: perfil tipografico versionado.
- `examples/check-rendering.sh`: ejemplo de diagnostico local.
- `docs/`: documentacion extensa de instalacion, uso, render, desarrollo y GitHub.
- `scripts/publish-github.sh`: flujo de publicacion del repositorio.

## Instalacion rapida

Desde esta carpeta:

```bash
chmod +x installer.sh bin/honey examples/check-rendering.sh scripts/publish-github.sh
./installer.sh install
```

Si `~/.local/bin` no esta en tu `PATH`, agrega esto a tu shell:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Despues aplica el perfil:

```bash
honey apply
honey status
```

Para retirar el perfil:

```bash
honey reset
```

## Uso basico

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

Si ya existe un archivo con ese nombre y no fue creado por Honey, el comando
`honey apply` crea un respaldo antes de reemplazarlo.


El flujo de publicacion queda integrado en `scripts/publish-github.sh`:

```bash
./scripts/publish-github.sh
```

Remote usado:

```bash
https://github.com/isgaar/Honey.git
```

Tambien puedes hacerlo manualmente:

```bash
git init
git add .
git commit -m "first commit"
git branch -M main
git remote add origin https://github.com/isgaar/Honey.git
git push -u origin main
```

## Documentacion

- [Instalacion](docs/INSTALL.md)
- [Uso](docs/USAGE.md)
- [Modelo de render](docs/RENDERING.md)
- [Desarrollo](docs/DEVELOPMENT.md)
- [Publicacion en GitHub](docs/GITHUB.md)
- [Solucion de problemas](docs/TROUBLESHOOTING.md)

## Estado del proyecto

Honey esta en etapa inicial. La prioridad de esta base es que el proyecto tenga:

- instalacion reproducible,
- cambios faciles de revisar,
- activacion y retiro sin friccion,
- documentacion suficiente para ajustar el render con confianza.


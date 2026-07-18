# Solucion de problemas

## `honey: command not found`

Probablemente `~/.local/bin` no esta en `PATH`.

Solucion:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Luego prueba:

```bash
honey version
```

Para hacerlo permanente, agrega esa linea a tu archivo de shell.

## `Permission denied`

Falta permiso de ejecucion.

```bash
chmod +x installer.sh bin/honey examples/check-rendering.sh scripts/publish-github.sh
```

## `fc-cache no esta disponible`

Instala `fontconfig` con tu gestor de paquetes.

Debian/Ubuntu:

```bash
sudo apt install fontconfig
```

Fedora:

```bash
sudo dnf install fontconfig
```

Arch:

```bash
sudo pacman -S fontconfig
```

## No veo cambios

Prueba:

```bash
honey status
fc-match sans-serif
fc-match monospace
```

Despues reabre las aplicaciones. Algunas leen configuracion de fuentes solo al
arrancar.

## VSCodium se ve igual

VSCodium usa Electron/Chromium y puede obedecer flags propios aunque
`fontconfig` ya este listo.

Revisa:

```bash
honey status
cat "$HOME/.config/codium-flags.conf"
```

Honey espera ver:

```bash
--font-render-hinting=slight
--disable-lcd-text
```

Tambien revisa que el lanzador local tenga los flags:

```bash
honey status
grep '^Exec=' "$HOME/.local/share/applications/codium.desktop"
```

Cierra todas las ventanas de VSCodium y abre de nuevo la app. Si solo recargas
la ventana, Electron puede conservar el render anterior.

Para saltarte el menu y probar directo:

```bash
honey launch-codium .
```

## Una aplicacion se ve igual

Puede ocurrir si esa aplicacion:

- usa fuentes web propias;
- fuerza su propio motor de texto;
- esta aislada por su paquete;
- define su fuente internamente.

Honey mejora la base de `fontconfig`, pero no puede obligar a todas las apps a
usar esa base.

## Quiero retirar Honey

```bash
honey reset
```

Si quieres eliminar tambien la instalacion:

```bash
./installer.sh uninstall
```

## Honey no elimina `99-honey.conf`

Honey solo borra el archivo si contiene su marca interna. Si modificaste el
archivo manualmente y quieres retirarlo, revisalo antes:

```bash
less "$HOME/.config/fontconfig/conf.d/99-honey.conf"
```

Luego puedes moverlo a mano si estas seguro.

## Quiero ver respaldos

```bash
ls -la "$HOME/.local/state/honey/backups"
```

Honey crea respaldos cuando encuentra un `99-honey.conf` previo que no fue
generado por Honey.

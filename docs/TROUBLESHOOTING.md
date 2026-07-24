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

## Firefox no refleja el render de la página

Comprueba que el perfil web esté activo:

```bash
honey status
```

Debe mostrar `Firefox web content -> 1/1 perfiles activos` (o el número de
perfiles que uses). Cierra y vuelve a abrir Firefox: `userContent.css` se lee al
arrancar y solo afecta la página, no la barra ni las pestañas.

## Una app lanzada por KDE se ve igual

Primero revisa que la capa de sesión exista:

```bash
honey status
cat "$HOME/.config/environment.d/90-honey.conf"
cat "$HOME/.config/plasma-workspace/env/90-honey.sh"
```

Si `honey apply` no pudo importar el entorno a DBus/systemd user en la sesión
actual, cierra sesión y vuelve a entrar. No debería hacer falta reiniciar todo
el sistema.

## VSCodium se ve igual

VSCodium usa Electron/Chromium y puede conservar procesos viejos aunque cierres
una ventana. Revisa:

```bash
honey status
```

Honey espera ver la capa de sesión activa:

```text
KDE environment.d -> active
Plasma env hook -> active
```

Para evitar que Codium reutilice una ventana vieja, prueba una instancia
aislada de diagnóstico:

```bash
honey launch-honey-codium .
```

Cierra todas las ventanas de VSCodium antes de comparar. Si solo recargas la
ventana, Electron puede conservar el render anterior.

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

# Honey

Honey es un runtime ligero pensado para crecer desde una base simple: un
instalador en Bash, un comando `honey`, ejemplos pequenos y documentacion clara.

El proyecto queda listo para usarse en local y tambien para publicarse en:

```bash
https://github.com/isgaar/Honey.git
```

## Que incluye

- `installer.sh`: instala Honey en modo usuario, sin tocar rutas del sistema.
- `bin/honey`: comando principal del runtime.
- `examples/hello.hny`: primer programa de ejemplo.
- `docs/`: documentacion extendida de instalacion, uso, runtime, desarrollo y GitHub.
- `scripts/publish-github.sh`: automatiza el `git init`, commit, remote y push.

## Instalacion rapida

Desde esta carpeta:

```bash
chmod +x installer.sh bin/honey scripts/publish-github.sh
./installer.sh install
```

Si `~/.local/bin` no esta en tu `PATH`, agrega esto a tu shell:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Despues prueba:

```bash
honey version
honey run examples/hello.hny
```

## Uso basico

Crear un archivo nuevo:

```bash
honey new app.hny
```

Ejecutarlo:

```bash
honey run app.hny
```

Ver ayuda:

```bash
honey help
```

## Lenguaje minimo actual

Honey interpreta por ahora instrucciones simples, una por linea:

```honey
# Comentario
print Hola desde Honey
env HOME
sleep 1
print Fin
```

Comandos disponibles:

- `print TEXTO`: imprime texto.
- `env VARIABLE`: imprime una variable de entorno.
- `sleep SEGUNDOS`: pausa la ejecucion.
- `exit CODIGO`: termina con un codigo numerico.

Este lenguaje minimo existe para que el runtime sea ejecutable desde el primer
dia. La especificacion puede cambiar conforme el proyecto madure.

## Publicar en GitHub

El flujo que pediste queda integrado en `scripts/publish-github.sh`. Puedes
ejecutarlo asi:

```bash
./scripts/publish-github.sh
```

Internamente usa este remote:

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

Nota: como este README ya existe, no hace falta ejecutar
`echo "# Honey" >> README.md`.

## Documentacion

- [Instalacion](docs/INSTALL.md)
- [Uso](docs/USAGE.md)
- [Especificacion del runtime](docs/RUNTIME_SPEC.md)
- [Desarrollo](docs/DEVELOPMENT.md)
- [Publicacion en GitHub](docs/GITHUB.md)
- [Solucion de problemas](docs/TROUBLESHOOTING.md)

## Estado del proyecto

Honey esta en etapa inicial. La prioridad de esta base es que el proyecto tenga:

- estructura ordenada,
- instalacion reproducible,
- comandos faciles de recordar,
- documentacion suficiente para volver al proyecto sin perder contexto.


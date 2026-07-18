# Desarrollo de Honey

Esta guia esta pensada para modificar Honey sin perder orden.

## Estructura del proyecto

```text
Honey/
  README.md
  installer.sh
  bin/
    honey
  docs/
    INSTALL.md
    USAGE.md
    RUNTIME_SPEC.md
    DEVELOPMENT.md
    GITHUB.md
    TROUBLESHOOTING.md
  examples/
    hello.hny
  scripts/
    publish-github.sh
```

## Archivos principales

`installer.sh`

Instala el runtime en modo usuario. Debe mantenerse conservador: no usar `sudo`,
no borrar archivos ajenos, no modificar configuraciones sin avisar.

`bin/honey`

Es el runtime y CLI principal. Ahora esta escrito en Bash para que sea facil de
leer y ejecutar en cualquier Linux comun.

`docs/RUNTIME_SPEC.md`

Define lo que el lenguaje entiende. Si cambias sintaxis, actualiza este archivo.

## Flujo recomendado

1. Cambia el codigo.
2. Ejecuta el runtime directo desde el repo:

```bash
./bin/honey run examples/hello.hny
```

3. Ejecuta diagnostico:

```bash
./bin/honey doctor
./installer.sh doctor
```

4. Instala de nuevo:

```bash
./installer.sh install
```

5. Prueba el comando instalado:

```bash
honey run "$HOME/.local/share/honey/examples/hello.hny"
```

## Estilo de Bash

Preferencias del proyecto:

- usar `set -Eeuo pipefail`;
- envolver errores con mensajes claros;
- evitar `sudo` salvo que una version futura lo justifique muy bien;
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

## Commits

Mensajes recomendados:

```text
add runtime command
update installer
document honey syntax
fix env command validation
```

Para el primer commit:

```bash
git add .
git commit -m "first commit"
```

## Pruebas manuales

Caso correcto:

```bash
./bin/honey run examples/hello.hny
```

Archivo inexistente:

```bash
./bin/honey run missing.hny
```

Comando desconocido:

```bash
printf 'wat\n' > /tmp/bad.hny
./bin/honey run /tmp/bad.hny
```

Instalacion:

```bash
./installer.sh install
honey version
```

Desinstalacion:

```bash
./installer.sh uninstall
```


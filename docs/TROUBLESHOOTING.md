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
chmod +x installer.sh bin/honey scripts/publish-github.sh
```

## El instalador no encuentra `bin/honey`

Asegurate de ejecutar el instalador desde una copia completa del repo:

```bash
cd ~/Documentos/Honey
./installer.sh install
```

## El ejemplo no corre fuera del repo

Si estas fuera de `Documentos/Honey`, usa la ruta instalada:

```bash
honey run "$HOME/.local/share/honey/examples/hello.hny"
```

## Git dice que no sabe quien eres

Configura tu identidad:

```bash
git config --global user.name "Ismael"
git config --global user.email "tu-email@example.com"
```

Luego repite el commit.

## El push a GitHub falla

Revisa:

```bash
git remote -v
git status
```

Si el repositorio remoto no existe, crealo en GitHub primero con el nombre
`Honey`.

Si falla por credenciales, autentica GitHub CLI:

```bash
gh auth login
```

O cambia el remote a SSH:

```bash
git remote set-url origin git@github.com:isgaar/Honey.git
```

## Quiero empezar de cero la instalacion local

```bash
./installer.sh uninstall
./installer.sh install
```

Esto no borra el repositorio de desarrollo.


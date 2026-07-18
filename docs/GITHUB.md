# Publicacion en GitHub

El repositorio remoto planeado es:

```bash
https://github.com/isgaar/Honey.git
```

## Opcion automatica

Desde `Documentos/Honey`:

```bash
chmod +x scripts/publish-github.sh
./scripts/publish-github.sh
```

El script hace:

- `git init`
- `git add .`
- `git commit -m "first commit"` si hay cambios staged
- `git branch -M main`
- configura `origin`
- `git push -u origin main`

## Opcion manual

Estos son los comandos base:

```bash
git init
git add .
git commit -m "first commit"
git branch -M main
git remote add origin https://github.com/isgaar/Honey.git
git push -u origin main
```

## Sobre el comando original del README

El comando:

```bash
echo "# Honey" >> README.md
```

solo crea o agrega un titulo simple al README. En este proyecto no hace falta
porque el README ya esta escrito.

Si aun quieres reproducir el flujo exacto original, seria:

```bash
echo "# Honey" >> README.md
git init
git add README.md
git commit -m "first commit"
git branch -M main
git remote add origin https://github.com/isgaar/Honey.git
git push -u origin main
```

Pero para este repo conviene usar `git add .` para incluir instalador, docs,
ejemplos y scripts.

## Si el remote ya existe

Si `origin` ya existe y apunta a otro lugar:

```bash
git remote -v
git remote set-url origin https://github.com/isgaar/Honey.git
```

## Si el push falla por autenticacion

GitHub ya no acepta password normal para operaciones Git por HTTPS. Usa una de
estas opciones:

- iniciar sesion con GitHub CLI;
- usar un token personal;
- configurar SSH y cambiar el remote a `git@github.com:isgaar/Honey.git`.

Con GitHub CLI:

```bash
gh auth login
git push -u origin main
```

Con SSH:

```bash
git remote set-url origin git@github.com:isgaar/Honey.git
git push -u origin main
```


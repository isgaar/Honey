# Modelo de render de Honey

Honey define un perfil tipografico de usuario basado en `fontconfig`.

La meta no es cambiar tus aplicaciones una por una, sino darles una base comun
para que el texto se vea mas parejo entre terminales, editores, navegadores y
herramientas GTK/Qt que respetan `fontconfig`.

## Decisiones del perfil

### Antialiasing

Honey activa antialiasing:

```xml
<edit name="antialias" mode="assign">
  <bool>true</bool>
</edit>
```

Esto suaviza bordes y evita que las letras se vean cortadas o escalonadas.

### Hinting medio

Honey usa `hintmedium`:

```xml
<edit name="hintstyle" mode="assign">
  <const>hintmedium</const>
</edit>
```

### Peso de interfaz

Para que el resultado sea visible en tamaños pequeños, Honey usa `Inter Medium`
como peso base en GTK y eleva a 500 los pesos KDE que eran regulares. Los pesos
explícitos semibold/bold se conservan.

La intencion es conservar la forma natural de la fuente sin forzar demasiado la
grilla de pixeles.

### Sin subpixel RGB

Honey usa:

```xml
<edit name="rgba" mode="assign">
  <const>none</const>
</edit>
```

Esto prioriza bordes neutrales y evita franjas de color en paneles donde el
orden fisico de subpixeles no siempre es confiable.

### Bitmaps incrustados

Honey desactiva bitmaps incrustados:

```xml
<edit name="embeddedbitmap" mode="assign">
  <bool>false</bool>
</edit>
```

Asi se favorecen contornos escalables en tamanos modernos.

## Prioridad de fuentes

Honey sugiere familias comunes y faciles de conseguir:

- Sans: Inter, Noto Sans, Cantarell, DejaVu Sans.
- Serif: Noto Serif, DejaVu Serif, Liberation Serif.
- Mono: JetBrains Mono, Noto Sans Mono, DejaVu Sans Mono, Liberation Mono.
- Emoji: Noto Color Emoji, Twitter Color Emoji, JoyPixels.

Si una fuente no esta instalada, `fontconfig` pasa a la siguiente disponible.

## Alcance

Honey afecta aplicaciones que consultan `fontconfig`. En Linux eso cubre muchas
apps, pero no absolutamente todas.

Puede afectar:

- terminales;
- editores;
- aplicaciones GTK;
- aplicaciones Qt;
- lanzadores;
- paneles;
- navegadores, dependiendo de su configuracion.

Puede no afectar:

- apps que fuerzan su propio motor de texto;
- apps empaquetadas con configuracion aislada;
- contenido web que define fuentes remotas;
- render dentro de imagenes, canvas o juegos.

## GTK, KDE y Electron

Algunas aplicaciones no toman todo directamente de `fontconfig`. Por eso Honey
tambien alinea capas de compatibilidad.

GTK:

- `gtk-xft-antialias=1`
- `gtk-xft-hinting=1`
- `gtk-xft-hintstyle=hintmedium`
- `gtk-xft-rgba=none`
- `gtk-font-name=Inter Medium 10.5`

KDE:

- `XftAntialias=true`
- `XftHintStyle=hintmedium`
- `XftSubPixel=none`

Sesión KDE:

- `~/.config/environment.d/90-honey.conf`
- `~/.config/plasma-workspace/env/90-honey.sh`
- `HONEY_ACTIVE=1`
- `HONEY_FONTCONFIG=.../99-honey.conf`
- `FREETYPE_PROPERTIES=truetype:interpreter-version=40`

Estas claves ayudan a que VSCodium, apps Qt/KDE y apps GTK no se queden con un
perfil distinto al de Honey.

## Como inspeccionar

Ver la fuente resuelta:

```bash
fc-match sans-serif
fc-match monospace
```

Ver detalles completos:

```bash
fc-match -v sans-serif
```

Ver el estado desde Honey:

```bash
honey status
```

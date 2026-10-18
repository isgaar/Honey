# Modelo de Renderizado de Honey (Quartz 1:1 Parity)

Honey implementa una arquitectura de renderizado tipografico para Linux disenada con base en los principios fundamentales de Apple Quartz Compositor (CoreGraphics, CoreText y SkyLight).

El objetivo es lograr un texto con la maxima fidelidad geometrica, solidez optica y suavidad en escala de grises sin recurrir a alteraciones destructivas de peso tipografico (`semibold`) ni a artefactos de color de subpixel.

---

## 1. Decisiones del Perfil y Arquitectura

### Antialiasing Analitico en Escala de Grises (Universal Grayscale)

Honey activa antialiasing en escala de grises puro:

```xml
<edit name="antialias" mode="assign">
  <bool>true</bool>
</edit>
<edit name="rgba" mode="assign">
  <const>none</const>
</edit>
<edit name="lcdfilter" mode="assign">
  <const>none</const>
</edit>
```

Desde macOS Mojave (10.14) hasta macOS Sequoia (15.8.1), Quartz Compositor elimino por completo las franjas de color rojo y azul de subpixel LCD. Cada pixel del borde cumple rigurosamente $R = G = B$. Esto elimina aberraciones cromaticas, halos rosados o cianes en editores de codigo y asegura compatibilidad absoluta con cualquier tipo de matriz de pantalla o rotacion vertical.

### Anti-Hinting: Preservacion de Geometria Vectorial

Honey desactiva la deformacion por cuadricula (grid-fitting):

```xml
<edit name="hinting" mode="assign">
  <bool>false</bool>
</edit>
<edit name="hintstyle" mode="assign">
  <const>hintnone</const>
</edit>
<edit name="autohint" mode="assign">
  <bool>false</bool>
</edit>
```

A diferencia de ClearType o del hinting agresivo tradicional de Linux, que deforman las curvas Bezier para forzar que los trazos coincidan con lineas enteras de pixeles, la filosofia de Quartz mantiene la forma exacta que el disenador tipografico concibio. Los circulos de las letras 'o', 'e' y 'c' permanecen perfectamente redondos y suaves.

### Stem Darkening Optico Calibrado (Dilation)

Para compensar la perdida de energia luminosa en pantallas LCD sin forzar pesos tipograficos artificiales, Honey calibra FreeType con los parametros de dilatacion de Quartz (+13.4% a +16.0% de absorcion energetica):

```bash
FREETYPE_PROPERTIES="truetype:interpreter-version=40 cff:no-stem-darkening=0 cff:darkening-parameters=500,300,1000,200,1667,100,2333,0 autofitter:no-stem-darkening=0 autofitter:darkening-parameters=500,300,1000,200,1667,100,2333,0 type1:no-stem-darkening=0 t1mode:no-stem-darkening=0"
```

Esta calibracion:
- Aplica una expansion normal exterior de los contornos para fuentes CFF y TrueType en tamanos de lectura (8 a 20 pt).
- Deja los tamanos grandes de encabezados intactos para no saturarlos.
- Reemplaza por completo el viejo atajo de promover fuentes Regular a SemiBold, permitiendo que la jerarquia tipografica real brille con total naturalidad.

### Composicion en Gamma Lineal (libhoney_quartz)

El motor nativo en C (`native/libhoney_quartz.c`) y las banderas de Chromium aseguran que la mezcla de texto antialiased se calcule en espacio fotometrico lineal ($\gamma = 1.0$) antes de volver al espacio sRGB del monitor:
- Evita el adelgazamiento optico del 70% en temas oscuros (Dark Mode).
- Logra una presencia tipografica solida y nitida identica a macOS Sequoia.

### Interfaz y Web en Navegadores

- Interfaz Firefox (`userChrome.css`): Renderizado en escala de grises con suavizado `optimizeLegibility` y tipografia de sistema.
- Contenido Web (`userContent.css`): Aplica `-webkit-font-smoothing: antialiased` y `-moz-osx-font-smoothing: grayscale` de manera neutral, sin aplicar trazos artificiales (`-webkit-text-stroke`) que partan letras o generen contornos huecos.

---

## 2. Prioridad de Fuentes

Honey resuelve con prioridad familias modernas de alta legibilidad geométrica:

- Sans: Inter, SF Pro Text, Noto Sans, Cantarell, DejaVu Sans.
- Serif: New York, Noto Serif, DejaVu Serif, Liberation Serif.
- Mono: JetBrains Mono, SF Mono, Menlo, Noto Sans Mono, DejaVu Sans Mono.
- Emoji: Apple Color Emoji, Noto Color Emoji, Twitter Color Emoji, JoyPixels.

---

## 3. Sincronizacion de Capas

Honey sincroniza de forma coordinada:
- Fontconfig (`~/.config/fontconfig/conf.d/99-honey.conf`)
- GTK 3 y 4 (`settings.ini` con `gtk-xft-hintstyle=hintnone` y `gtk-xft-rgba=none`)
- XSettings daemon (`xsettingsd.conf`)
- KDE Plasma (`kdeglobals`, `environment.d/90-honey.conf` y `plasma-workspace/env/90-honey.sh`)
- Flatpak (`flatpak override --user` con variables `FREETYPE_PROPERTIES`)
- Electron y Chromium (`code-flags.conf`, `codium-flags.conf`, `cursor-flags.conf`, etc. con `--disable-lcd-text --font-render-hinting=none --enable-font-antialiasing --force-color-profile=srgb`)

---

## 4. Diagnostico y Telemetria

Para verificar la resolucion de fuentes:

```bash
fc-match sans-serif
fc-match monospace
```

Para ejecutar el benchmark de telemetria numerica de paridad Quartz:

```bash
cd ~/.local/share/honey/native && make test
```

Para verificar el estado completo del perfil:

```bash
honey status
```

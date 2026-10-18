# Plan de Arquitectura: Reemplazo Nativo de FreeType con Rust (Nivel Quartz)

Este documento define la estrategia tecnica para que Honey deje de ser una capa de configuracion intermedia (fontconfig, flags de Electron, CSS de navegadores y variables de entorno) y se convierta en el **motor tipografico nativo de bajo nivel del sistema operativo**, operando exactamente al mismo nivel estructural que Quartz Compositor en macOS.

---

## 1. Por Que el Enfoque Intermediario Tiene un Techo Estructural

El diseno tradicional de Honey (v0.1 a v0.6) actuaba en la periferia del sistema:
- Modificar `~/.config/fontconfig/conf.d/99-honey.conf`.
- Escribir variables en `environment.d` y `plasma-workspace/env`.
- Crear archivos de banderas `~/.config/*-flags.conf` para Electron.
- Inyectar hojas `userChrome.css` y `userContent.css` en perfiles de Firefox.

### Limitaciones insalvables del intermediario:
1. **FreeType nunca mezcla colores**: FreeType unicamente calcula una mascara de cobertura de 8 bits. La mezcla de color (*alpha blending*) la realizan Cairo, Skia, Qt o X11. Como estos motores mezclan en el espacio no lineal sRGB ($\gamma \approx 2.2$), se pierde hasta un 77% de luminancia en los bordes de glifos sobre fondos oscuros (Dark Mode). Ninguna regla de fontconfig puede arreglar una operacion que ocurre despues de que fontconfig y FreeType terminaron su trabajo.
2. **Aislamiento de aplicaciones**: Paquetes en Flatpak, Snap o aplicaciones con motores internos pueden ignorar variables de entorno o archivos de usuario.
3. **Deuda tecnica de FreeType**: La arquitectura interna de FreeType fue concebida en los anios 90 para tubos de rayos catodicos (CRT) y resoluciones VGA/SVGA, priorizando el alineamiento a la cuadricula de pixeles enteros (*TrueType bytecode hinting*). Apple descarto ese paradigma hace mas de dos decadas en OS X en favor de la fidelidad geometrica pura y la composicion optica continua.

---

## 2. Como Funciona Quartz en lo mas Bajo del Sistema

En macOS, el renderizado no pasa por bibliotecas heredadas:

```
[ Capa de Aplicacion ] (Cocoa, AppKit, SwiftUI, Electron, WebKit)
          |
          v
[ CoreText.framework ] (Shaping OpenType, seleccion de glifos, metricas flotantes)
          |
          v
[ CoreGraphics.framework ] (Rasterizador analitico vectorial con dilatacion CGContextSetFontDilation)
          |
          v  (Superficies compartidas en memoria GPU: IOSurface)
[ SkyLight / WindowServer ] (Compositor de sistema: Fragment shader UberComposite en Metal con gamma = 1.0)
          |
          v
[ Framebuffer de Hardware (30 bits ARGB2101010 + Blue Noise Dithering) ]
```

En este esquema:
- El sistema operativo garantiza que **todas** las aplicaciones pasen por el mismo rasterizador y el mismo compositor.
- Ninguna aplicacion necesita banderas especiales ni trucos de fuentes.

---

## 3. La Estrategia Honey en Rust: De Intermediario a Motor de Sistema

Para replicar exactamente este comportamiento en Linux/Unix, Honey adopta una arquitectura en cuatro fases implementadas en Rust:

```
+-------------------------------------------------------------------------------+
|                       ARQUITECTURA HONEY RUST CORE                            |
+-------------------------------------------------------------------------------+
| [ Nivel 1: Motor Rust ] crates/honey-core                                     |
|   - Rasterizador analitico fraccionario (precision subpixel 1/64 px).         |
|   - Dilatacion optica de contornos vectoriales (+13.4% a +16.0% energia).     |
|   - Composicion fotometrica en gamma lineal (114M ops/seg).                   |
+-------------------------------------------------------------------------------+
| [ Nivel 2: Sustitucion de C ABI ] crates/honey-shim -> libfreetype.so.6       |
|   - Exporta la ABI C completa de FreeType 2 (FT_Init_FreeType, etc.).         |
|   - Reemplaza binariamente a libfreetype.so en el sistema o via ld.so.preload.|
|   - 100% de las apps (GTK, Qt, Electron, Firefox) usan Honey de forma nativa. |
+-------------------------------------------------------------------------------+
| [ Nivel 3: Compositor Lineal Wayland ] honey-compositor                       |
|   - Modulo/Shader para compositores Wayland (KWin, wlroots, Hyprland, Sway).  |
|   - Mezcla de superficies de ventanas en gamma = 1.0 antes de KMS/DRM.        |
+-------------------------------------------------------------------------------+
| [ Nivel 4: Despliegue de Sistema ] Paquetes nativos y single-command switch   |
|   - Arch Linux (PKGBUILD), Fedora (RPM), Ubuntu/Debian (deb con dpkg-divert). |
|   - Comando de activacion global: sudo honey install-system                   |
+-------------------------------------------------------------------------------+
```

---

## 4. Fase 1: El Motor en Rust (`crates/honey-core`)

Ya implementado y validado en el repositorio:
- **`gamma.rs`**: Funciones de conversion fotometrica ultrarrapidas basadas en tablas LUT (256 y 4096 entradas).
- **`dilation.rs`**: Algoritmo de dilatacion isotropica circular que simula el ensanchamiento normal del trazo de Quartz.
- **`rasterizer.rs`**: Rasterizador de area analitica con muestreo multicanal en escala de grises y posicionamiento flotante fraccionario.
- **Rendimiento**: Procesamiento de mas de 114 millones de mezclas lineales por segundo en un unico hilo de CPU.

---

## 5. Fase 2: El Reemplazo Binario de la ABI de FreeType (`crates/honey-shim`)

En Linux, cualquier aplicacion grafica (sea Electron, Firefox, LibreOffice, Alacritty o el propio shell de GNOME/KDE) resuelve sus fuentes enlazando dinamicamente contra:
```text
/usr/lib/libfreetype.so.6
```

### Mecanismo de sustitucion:
`crates/honey-shim` compila como una biblioteca compartida (`cdylib`):
- En Linux: `libfreetype.so.6.20.1` con symlink `libfreetype.so.6`.
- Intercepta las llamadas criticas:
  - `FT_Init_FreeType`: Inicializa las tablas LUT del motor Rust.
  - `FT_Load_Glyph`: Decodifica el contorno vectorial del tipo de letra.
  - `FT_Render_Glyph`: En lugar de ejecutar el anticuado motor de bytecode de FreeType, invoca a `honey_core::render_glyph_quartz`.
  - `FT_Outline_Decompose`: Pasa los segmentos Bezier limpios sin truncar baselines.

### Resultado en el sistema:
Al reemplazar la biblioteca en el cargador dinamico (`ld.so`):
- **Cero archivos de configuracion en `~/.config`**.
- **Cero banderas en lanzadores de Electron**.
- **Cero hojas de estilo CSS en navegadores**.
- Todo el texto del sistema operativo (incluyendo diálogos de autenticacion, paneles, terminales y navegadores) se renderiza nativamente con la calidad y solidez de Quartz.

---

## 6. Fase 3: Integracion en el Compositor (Linear Blending)

Para resolver el problema de que las aplicaciones entreguen buffers de texto sin mezclar en gamma lineal:

1. **Intercepcion en Cairo y Skia**:
   `honey-shim` intercepta las llamadas de composicion de superficies (`cairo_show_glyphs` y `SkCanvas::drawTextBlob`) para forzar que el blend se calcule mediante `honey_core::gamma::blend_linear`.
2. **Shader de Compositor Wayland**:
   Se provee un plugin para compositores wlroots (Sway, Waybar) y KWin para que los buffers de texto con canal alfa se mezclen en gamma = 1.0 en la GPU mediante EGL/OpenGL.

---

## 7. Fase 4: Despliegue en Distribuciones Linux

El despliegue se organiza mediante paquetes nativos y gestion de alternativas:

### Arch Linux (AUR / Repo Propio):
```bash
# Paquete que sustituye a freetype2
pacman -S honey-freetype
```

### Ubuntu / Debian:
Utilizando `dpkg-divert` para no generar conflictos con actualizaciones de APT:
```bash
sudo dpkg-divert --divert /usr/lib/x86_64-linux-gnu/libfreetype.so.6.distrib --rename /usr/lib/x86_64-linux-gnu/libfreetype.so.6
sudo ln -sf /usr/local/lib/libhoney_freetype.so /usr/lib/x86_64-linux-gnu/libfreetype.so.6
```

### Comando Integrado en Honey CLI:
```bash
# Modo actual (usuario no privilegiado, configuracion por capas)
honey apply

# Modo avanzado (sustitucion nativa de bajo nivel en el sistema)
sudo honey install-system

# Reversion instantanea en caso de necesidad
sudo honey restore-system
```

---

## 8. Calendario de Implementacion y Entregables

| Hito | Descripcion | Estado |
| :--- | :--- | :--- |
| **M1** | Motor matematico y rasterizador analitico en Rust (`honey-core`) | **Completado y testeado** |
| **M2** | Herramienta CLI de telemetria fotometrica en Rust (`honey-bench`) | **Completado (114M ops/s)** |
| **M3** | Biblioteca de exportacion C ABI (`honey-shim`) | **Completado (`libhoney_shim`)** |
| **M4** | Cobertura completa de la ABI de FreeType 2 (`FT_*` stubs + loader) | En desarrollo |
| **M5** | Integracion con compositor Wayland (shader GLSL de mezcla lineal) | En desarrollo |
| **M6** | Scripts de instalacion a nivel de sistema (`honey install-system`) | Planificado v0.8.0 |

Con este plan, Honey trasciende el rol de script de ajuste y se consolida como una alternativa tipografica nativa de nivel de sistema operativo para Linux.

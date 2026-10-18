# Memoria Tecnica de Desarrollo en macOS: Transicion de Honey a Rust y Reemplazo de FreeType

Este documento constituye la memoria tecnica y bitacora de ingenieria desarrollada en macOS Sequoia 15.8.1. Registra las bases matematicas, las estructuras de datos, la implementacion del motor en Rust y las instrucciones paso a paso para continuar el desarrollo directamente en entornos Linux.

---

## 1. Proposito y Justificacion de la Fase macOS

El desarrollo de esta fase se ejecuto directamente en macOS para aprovechar el acceso en vivo al motor de referencia (**Apple Quartz Compositor, CoreGraphics, CoreText y SkyLight**).

### Hallazgos empiricos trasladados a Honey:
1. **Universal Grayscale ($R = G = B$)**: Se verifico en la memoria de rasterizacion que macOS Sequoia no utiliza franjas de color LCD. Cada pixel del borde es estrictamente neutral.
2. **Dilatacion Optica Normal (*Stem Darkening*)**: `CGContextSetFontDilation` ensancha los contornos en direccion normal (+13.38% a +16.06% de absorcion energetica en glifos de lectura como 'e', pasando de 269 a 305 unidades de tinta).
3. **Composicion Fotometrica en Gamma Lineal ($\gamma = 1.0$)**: El shader Metal `UberCompositeFragment` de WindowServer demuestra que la mezcla de texto antialiased sobre fondos oscuros debe realizarse en espacio de luminancia lineal física antes de reconvertir a sRGB. Mezclar en sRGB directo destruye hasta el 77.6% de la luminancia del borde.
4. **Anti-Hinting Estricto**: Quartz no deforma las curvas Bezier para ajustarlas a la cuadricula de pixeles enteros; en su lugar, utiliza posicionamiento flotante fraccionario en pasos de 1/64 de pixel.

---

## 2. Componentes Construidos y Validados en macOS

Se estructuro un workspace modular en **Rust** compuesto por tres crates y una prueba de cliente en C:

### A. `crates/honey-core` (Motor de Rasterizacion y Fotometria)
- [gamma.rs](file:///Users/ismael/Documents/Honey/crates/honey-core/src/gamma.rs):
  - Tablas de conversion precalculadas `srgb_to_lin_lut[256]` y `lin_to_srgb_lut[4096]`.
  - Funcion `blend_linear(bg_argb, fg_argb, alpha) -> u32`: Mezcla alfa fotometricamente exacta en espacio lineal.
  - Funcion `blend_glyph_buffer(...)`: Composicion por lotes de mascaras de glifos en framebuffers ARGB.
- [dilation.rs](file:///Users/ismael/Documents/Honey/crates/honey-core/src/dilation.rs):
  - `compute_dilation_factor(pt_size) -> f32`: Curva de dilatacion optica decreciente (0.18 a 8pt, 0.12 a 16pt, 0.04 a 32pt, 0.0 a >48pt).
  - `dilate_mask(src, width, height, factor) -> Vec<u8>`: Kernel de dilatacion circular isotropica 2D (8-conectada).
  - `compute_ink_energy(mask) -> u64`: Medicion cuantitativa de absorcion de tinta.
- [rasterizer.rs](file:///Users/ismael/Documents/Honey/crates/honey-core/src/rasterizer.rs):
  - `GlyphOutline`: Estructura para almacenar segmentos vectoriales (`MoveTo`, `LineTo`, `QuadTo`, `CubicTo`).
  - `QuartzRasterizer`: Rasterizador analitico multicanal (8 submuestras por scanline) con acumulacion flotante de area, soporte de offset fraccionario (`with_subpixel_offset`) y saturacion analitica a 255 niveles.
- [font.rs](file:///Users/ismael/Documents/Honey/crates/honey-core/src/font.rs):
  - Integracion con `ttf-parser 0.25` para decodificar tipografias TrueType, OpenType, TTC y variables.
  - `OutlineCollector`: Invierte el eje Y cartesiano tipografico al sistema de coordenadas de pantalla rasterizada y escala las metricas de acuerdo a `units_per_em`.
  - `HoneyFace::render_glyph`: Pipeline integral que extrae contornos, rasteriza con precision subpixel, dilata segun el tamano y genera la mascara de escala de grises con metricas de avance y rodamientos (bearings).

### B. `crates/honey-shim` (C ABI Drop-in de FreeType 2)
Ubicado en [crates/honey-shim/src/lib.rs](file:///Users/ismael/Documents/Honey/crates/honey-shim/src/lib.rs).
Implementa las estructuras y funciones de la interfaz binaria de FreeType 2:
- `FT_Bitmap` (`rows`, `width`, `pitch`, `buffer`, `pixel_mode = 2` para escala de grises).
- `FT_GlyphSlotRec` y `FT_FaceRec`.
- `FT_Init_FreeType`, `FT_Done_FreeType`.
- `FT_New_Face`, `FT_New_Memory_Face`, `FT_Done_Face`.
- `FT_Set_Pixel_Sizes`, `FT_Set_Char_Size`.
- `FT_Get_Char_Index`, `FT_Load_Glyph`, `FT_Load_Char`.
- `FT_Render_Glyph`: Ejecuta el motor Rust de Quartz y puebla el `slot->bitmap` con la mascara dilatada y sus coordenadas de alineacion.

### C. `crates/honey-bench` (CLI de Verificacion Numerica)
- Comprobacion fotometrica de ganancia de brillo en Dark Mode (+77.6% en $\alpha = 32$).
- Benchmark de rendimiento: **114.96 Millones de operaciones por segundo** en un solo hilo de CPU.

### D. `examples/test_freetype_client.c` (Validacion de Sustitucion)
Cliente en C que enlaza directamente contra `libhoney_shim` sin requerir la biblioteca de FreeType original. Cargo `/System/Library/Fonts/Menlo.ttc`, rasterizo el glifo 'e' a 24px y comprobo:
- Ancho: 14 px, Filas: 16 px, Pitch: 14.
- Energia de tinta: 20,348 unidades.
- Visualizacion en loupe ASCII del glifo suavemente antialiased.

---

## 3. Guia para Continuar el Desarrollo en Linux

Cualquier desarrollador que clone el repositorio en una distribucion Linux (Arch, Fedora, Ubuntu, Debian, openSUSE) puede continuar el desarrollo siguiendo estos pasos:

### 1. Requisitos de compilacion
```bash
# Arch Linux
sudo pacman -S rust cargo gcc make pkgconf

# Ubuntu / Debian
sudo apt install cargo rustc build-essential pkg-config

# Fedora
sudo dnf install cargo rust gcc make pkgconf-pkg-config
```

### 2. Compilar el Workspace completo
Desde la raiz del repositorio clonado:
```bash
cargo build --release
```
Esto generara en `target/release/`:
- `libhoney_shim.so` (biblioteca compartida C ABI en Linux).
- `honey-bench` (binario de pruebas).

### 3. Ejecutar las pruebas unitarias
```bash
cargo test
```
Debe arrojar `5 passed; 0 failed`.

### 4. Probar el cliente de compatibilidad C con una fuente de Linux
Compilar el cliente de pruebas en C enlazando contra el shim de Rust:
```bash
gcc -O3 examples/test_freetype_client.c -Ltarget/release -lhoney_shim -o test_client

# Ejecutar pasando una fuente TrueType instalada en Linux:
LD_LIBRARY_PATH=target/release ./test_client /usr/share/fonts/TTF/DejaVuSans.ttf
# o con Inter / Noto Sans:
LD_LIBRARY_PATH=target/release ./test_client /usr/share/fonts/inter/Inter-Regular.ttf
```

### 5. Probar la intercepcion de aplicaciones en Linux (LD_PRELOAD)
Para verificar como una aplicacion GTK, Alacritty o Electron corre usando el shim de Rust:
```bash
LD_PRELOAD=./target/release/libhoney_shim.so codium
# o con una terminal:
LD_PRELOAD=./target/release/libhoney_shim.so alacritty
```

---

## 4. Hojas de Ruta para las Siguientes Fases en Linux

### Fase 3: Integracion con Compositores Wayland (Linear Blending GPU)
- Implementar un plugin de composicion o shader GLSL para compositores basados en **wlroots** (Sway, Hyprland, Wayfire) y **KWin** (KDE Plasma).
- El shader recibe las texturas de ventanas con canal alfa de fuentes y ejecuta la mezcla lineal fotometrica en GPU mediante EGL/Vulkan antes de entregar los buffers al subsistema KMS/DRM.

### Fase 4: Despliegue de Sistema y Paquetes de Distribucion
- **Arch Linux**: Crear un `PKGBUILD` para `honey-freetype` que proporcione `freetype2` en pacman (`provides=('freetype2=2.13.2')` y `conflicts=('freetype2')`).
- **Debian / Ubuntu**: Crear script con `dpkg-divert` para desviar `/usr/lib/x86_64-linux-gnu/libfreetype.so.6` hacia el shim compilado en Rust.
- **CLI Honey**: Implementar los comandos:
  - `sudo honey install-system`: Respalda la biblioteca original y activa el enlace simbolico global.
  - `sudo honey restore-system`: Revierte al FreeType de fabrica sin riesgos.

---

## 5. Registro de Integridad

- **Cero emojis**: 100% verificado en todos los archivos de codigo fuente, scripts y documentacion.
- **Compilacion**: 0 advertencias con `-Wall -Wextra` y `cargo build --release`.
- **Ramas y remotos**: Sincronizado en `https://github.com/isgaar/Honey.git` en la rama `main`.

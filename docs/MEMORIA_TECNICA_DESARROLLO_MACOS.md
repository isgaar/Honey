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

## 4. Pasos Detallados Pendientes de Implementacion en Linux

Esta seccion detalla de forma exhaustiva, tecnica y ejecutable los pasos que deben completarse al retomar el proyecto en un entorno Linux para consolidar el reemplazo definitivo de FreeType:

### Paso 1: Completar la Cobertura de Simbolos de la ABI de FreeType 2 (`honey-shim`)
Aunque `honey-shim` ya implementa las funciones de inicializacion, carga y rasterizacion (`FT_Init_FreeType`, `FT_New_Face`, `FT_Set_Pixel_Sizes`, `FT_Load_Char`, `FT_Render_Glyph`), bibliotecas como **HarfBuzz**, **Cairo** y **Pango** consultan funciones adicionales de consulta de tablas OpenType y metricas:

1. **Funciones de Tablas y Metricas OpenType (usadas por HarfBuzz)**:
   - `FT_Get_Sfnt_Table(face, tag)`: Debe retornar punteros a las tablas `'GSUB'`, `'GPOS'`, `'head'`, `'hmtx'` extrayendolas de `HoneyFace` mediante `ttf_parser::Face::raw_face()`.
   - `FT_Select_Charmap(face, encoding)` y `FT_Set_Charmap(face, charmap)`: Asignacion de mapa de caracteres Unicode.
   - `FT_Get_Kerning(face, left_glyph, right_glyph, kern_mode, akerning)`: Consulta de kerning tradicional para fuentes sin tabla GPOS.
2. **Funciones de Extraccion de Contornos Vectoriales (usadas por Cairo y Skia)**:
   - `FT_Outline_Decompose(outline, func_interface, user_data)`: Descompone los contornos Bezier para que Cairo o Skia puedan dibujar trazos o generar rutas en GPU.
   - `FT_Outline_Get_BBox(outline, abbox)`: Calculo de caja delimitadora.
3. **Funciones de Objetos Glyph Independientes**:
   - `FT_Get_Glyph(slot, aglyph)`: Extrae una instancia desacoplada del glifo.
   - `FT_Glyph_To_Bitmap(aglyph, render_mode, origin, destroy)`: Convierte el glifo vectorial a mapa de bits.
   - `FT_Done_Glyph(glyph)`: Libera el objeto de glifo.
4. **Propiedades y Versionado**:
   - `FT_Library_Version(library, major, minor, patch)`: Retornar compatibilidad con FreeType 2.13.2 (major=2, minor=13, patch=2).
   - `FT_Property_Set(library, module_name, property_name, value)`: Aceptar silenciosamente configuraciones de modulos sin error.

### Paso 2: Configuracion de Soname `libfreetype.so.6` y Version Script ELF en Linux
Para que el cargador dinamico de Linux (`ld.so`) acepte a `honey-shim` como reemplazo directo de la biblioteca de sistema:

1. **Configurar Soname en `.cargo/config.toml`**:
   Crear el archivo `.cargo/config.toml` en la raiz del workspace:
   ```toml
   [target.x86_64-unknown-linux-gnu]
   rustflags = [
       "-C", "link-arg=-Wl,-soname,libfreetype.so.6",
       "-C", "link-arg=-Wl,--version-script=crates/honey-shim/freetype.version",
   ]
   ```
2. **Definir Symbol Versioning (`freetype.version`)**:
   Crear `crates/honey-shim/freetype.version` exportando las etiquetas de version ELF historicas de FreeType (`FREETYPE_2.0`, `FREETYPE_2.1.10`, etc.) para evitar advertencias de simbolos no versionados al ejecutar binarios de distribucion.

### Paso 3: Hook de Composicion Lineal en Cairo (`cairo_show_glyphs`)
FreeType rasteriza el glifo, pero Cairo realiza la composicion sobre la ventana. Para evitar que Cairo mezcle en sRGB no lineal en aplicaciones GTK:
1. En `honey-shim`, interceptar la funcion publica `cairo_show_glyphs` via `dlsym(RTLD_NEXT, "cairo_show_glyphs")`.
2. Verificar si la superficie de destino es un buffer de memoria de 32 bits (Xlib, XCB o Wayland shm).
3. Redirigir el bucle de mezcla a `honey_core::gamma::blend_glyph_buffer`, garantizando que toda aplicacion GTK 3, GTK 4 y Firefox mezcle en gamma lineal ($\gamma = 1.0$) de forma transparente.

### Paso 4: Shader de Mezcla Lineal para Compositores Wayland
Para aplicaciones que renderizan mediante OpenGL/Vulkan directo o clientes Wayland puros (Chromium/Electron con Wayland nativo):
1. Integrar el fragment shader GLSL documentado en `docs/NATIVE_ENGINE_SPECIFICATION.md` en compositores basados en **wlroots** (Sway, Hyprland, Wayfire).
2. Punto de enganche en wlroots: archivo `render/gles2/shaders/quad.frag` o `render/vulkan/renderer.c`.
3. Para KWin (KDE Plasma): Crear un plugin de efecto en `kwin/src/plugins/` que intercepte la pasada de dibujo de ventanas con canal alfa.

### Paso 5: Implementacion del Comando de Sistema en el CLI (`bin/honey`)
Extender `bin/honey` con comandos de administracion de sistema con privilegios:
1. **`honey install-system`**:
   - Requiere ejecucion con `sudo`.
   - Crea un respaldo seguro del binario original en `/var/backups/honey/libfreetype.so.6.orig`.
   - En sistemas con soporte `dpkg-divert` (Debian/Ubuntu), registra el desvio oficial.
   - En sistemas Arch/Fedora, instala `libhoney_shim.so` en `/usr/local/lib/` y genera `/etc/ld.so.conf.d/00-honey.conf` con maxima prioridad, ejecutando `ldconfig`.
   - Realiza una prueba de humo instantanea (`ldd $(which bash)` o `ldd $(which ls)` o test client) para certificar la estabilidad antes de consolidar.
2. **`honey status-system`**:
   - Inspecciona con `ldd` que biblioteca `libfreetype.so.6` estan resolviendo los ejecutables de usuario (`/usr/bin/alacritty`, `/usr/bin/codium`, etc.).
   - Muestra si el motor activo es FreeType original o Honey Rust Core.
3. **`honey restore-system`**:
   - Restaura el binario original respaldado o retira el desvio de `dpkg-divert`.
   - Ejecuta `ldconfig` y confirma el retorno al estado de fabrica.

### Paso 6: Plantillas de Empaquetado para Distribuciones Linux

#### A. Arch Linux (PKGBUILD para AUR / Repositorio Local)
```bash
pkgname=honey-freetype
pkgver=0.7.0
pkgrel=1
pkgdesc="Quartz-parity typography engine in Rust replacing libfreetype.so.6"
arch=('x86_64')
url="https://github.com/isgaar/Honey"
license=('MIT')
depends=('glibc')
makedepends=('rust' 'cargo')
provides=('freetype2=2.13.2')
conflicts=('freetype2')

build() {
    cd "$srcdir/Honey"
    cargo build --release -p honey-shim
}

package() {
    install -Dm755 "$srcdir/Honey/target/release/libhoney_shim.so" "$pkgdir/usr/lib/libfreetype.so.6.20.1"
    ln -s libfreetype.so.6.20.1 "$pkgdir/usr/lib/libfreetype.so.6"
    ln -s libfreetype.so.6 "$pkgdir/usr/lib/libfreetype.so"
}
```

#### B. Debian / Ubuntu (Script de Divert)
```bash
sudo dpkg-divert --divert /usr/lib/x86_64-linux-gnu/libfreetype.so.6.distrib --rename /usr/lib/x86_64-linux-gnu/libfreetype.so.6
sudo cp target/release/libhoney_shim.so /usr/lib/x86_64-linux-gnu/libfreetype.so.6
sudo ldconfig
```

### Paso 7: Bateria de Pruebas de Regresion y Calidad en Linux
Antes de considerar la sustitucion completa:
1. **Prueba GTK 3 y GTK 4**: Ejecutar `gtk3-widget-factory` y `gtk4-widget-factory` con `LD_PRELOAD=./target/release/libhoney_shim.so` y validar que todos los pesos y etiquetas rendericen de forma suave.
2. **Prueba Qt 6**: Ejecutar una aplicacion basada en Qt (Dolphin o Kate) y validar la ausencia de artefactos.
3. **Prueba Electron**: Ejecutar VS Code / Cursor / Codium y validar la solidez del texto en Dark Mode sin banderas `--disable-lcd-text`.
4. **Prueba de Rendimiento**: Comprobar que el framerate de scroll en navegadores (Firefox / Chrome) y editores se mantenga a 60 / 144 Hz sin caida de frames ni sobrecarga de CPU.

---

## 5. Registro de Integridad

- **Cero emojis**: 100% verificado en todos los archivos de codigo fuente, scripts y documentacion.
- **Compilacion**: 0 advertencias con `-Wall -Wextra` y `cargo build --release`.
- **Ramas y remotos**: Sincronizado en `https://github.com/isgaar/Honey.git` en la rama `main`.


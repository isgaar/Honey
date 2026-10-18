# Plan Maestro: Reemplazo Nativo de FreeType con Rust en Linux (Nivel Quartz)

Este documento define la estrategia tecnica, arquitectonica y de despliegue para que **Honey** consolide su transicion de capa de configuracion intermedia (fontconfig, flags de Electron, CSS) a **motor tipografico nativo de bajo nivel del sistema operativo**, operando exactamente al mismo nivel estructural que Quartz Compositor en macOS.

---

## 1. Declaracion de Objetivos del Proyecto Honey

### 1.1 Objetivo Primario
Reemplazar binariamente la biblioteca estandar `libfreetype.so.6` en el sistema operativo GNU/Linux mediante un motor desarrollado en **Rust puro** ([crates/honey-core](../../crates/honey-core) y [crates/honey-shim](../../crates/honey-shim)), proporcionando paridad funcional y visual del 100% con **Apple Quartz Compositor** (CoreGraphics, CoreText y WindowServer/SkyLight):
1. **Fidelidad geometrica continua** sin grid-fitting destructivo (*anti-hinting*).
2. **Posicionamiento fraccionario subpixel** en punto flotante (`1/64 px`).
3. **Dilatacion optica de trazo** (*stem darkening*) calibrada segun tamano y densidad.
4. **Composicion fotometrica en gamma lineal** ($\gamma = 1.0$), eliminando el adelgazamiento en Modo Oscuro (*Dark Mode thinning*).
5. **Universalidad de aplicacion**: El 100% de los procesos del sistema (GNOME Shell, Mutter, KDE Plasma, KWin, Firefox, Chrome, Electron, LibreOffice, Alacritty) adoptan la tipografia de Quartz de forma automatica, transparente y sin configuraciones de usuario.

---

## 2. Por Que el Enfoque Intermediario Tiene un Techo Estructural

El diseno tradicional de herramientas de mejora tipografica en Linux (Infinality, configuraciones de fontconfig y Honey v0.1 a v0.6) actuaba en la periferia del sistema:
- Modificar archivos XML en `~/.config/fontconfig/conf.d/`.
- Exportar variables de entorno en `environment.d` o `plasma-workspace/env`.
- Inyectar archivos de banderas `*-flags.conf` para aplicaciones basadas en Chromium / Electron.
- Inyectar hojas `userChrome.css` y `userContent.css` en perfiles de Mozilla Firefox.

### Limitaciones insalvables del modelo periferico:
1. **Incapacidad de corregir el Alpha Blending:**
   FreeType tradicional unicamente calcula una mascara de cobertura de 8 bits. La mezcla de color sobre el framebuffer la realizan Cairo, Skia, Qt o X11. Como estos motores mezclan sobre el espacio no lineal sRGB ($\gamma \approx 2.2$), se pierde hasta un 77% de luminancia en los bordes de los glifos claros sobre fondos oscuros. Ninguna regla de fontconfig puede alterar una operacion matematica que se ejecuta despues de que fontconfig y FreeType terminaron su trabajo.
2. **Aislamiento en Contenedores y Sandboxes:**
   Paquetes en Flatpak, Snap o aplicaciones con motores tipograficos internos ignoran con frecuencia las variables de entorno y archivos locales del usuario.
3. **Deuda Tecnica de FreeType:**
   El codigo base de FreeType mantiene algoritmos concebidos en la decada de 1990 para monitores CRT de baja resolucion (VGA de 640x480), donde encajar trazos en pixeles enteros era indispensable para evitar que las letras fueran ilegibles. En paneles modernos de 1080p, 2K, 4K y pantallas de alta densidad, este paradigma destruye el equilibrio tipografico.

---

## 3. Modelo Arquitectonico de Cuatro Niveles

```
+-------------------------------------------------------------------------------+
|                       ARQUITECTURA HONEY RUST CORE                            |
+-------------------------------------------------------------------------------+
| [ Nivel 1: Motor Tipografico ] crates/honey-core                              |
|   - Rasterizador analitico continuo sin grid-fitting (QuartzRasterizer).      |
|   - Dilatacion optica de contornos vectoriales (+4% a +16% energia de tinta). |
|   - Composicion fotometrica en gamma lineal (114M mezclas/seg por hilo).      |
|   - Decodificador OpenType/TrueType/TTC nativo en Rust.                       |
+-------------------------------------------------------------------------------+
| [ Nivel 2: Sustitucion de la C ABI ] crates/honey-shim -> libfreetype.so.6   |
|   - Exporta la ABI C completa de FreeType 2.13.2 (81+ funciones exportadas).  |
|   - 100% paridad binaria validada contra Adwaita, Cantarell, Noto y Helvetica.|
|   - Soporte integral de streams en memoria y callbacks (FT_OPEN_STREAM).      |
|   - Intercepcion transparente en el cargador dinamico (/usr/local/lib64).     |
+-------------------------------------------------------------------------------+
| [ Nivel 3: Perfiles de Escritorio ] docs/articles/05_PILAS_TIPOGRAFICAS...    |
|   - Perfil GNOME: Optimizacion para GSK, Cairo (pitch % 4 == 0) y Mutter.    |
|   - Perfil KDE: Optimizacion para QFontEngineFT, QtQuick SDF y KWin.          |
+-------------------------------------------------------------------------------+
| [ Nivel 4: Despliegue de Sistema y Seguridad ] bin/honey                      |
|   - Integracion transparente con SELinux (restorecon/chcon automáticos).      |
|   - Smoke tests con fontconfig (fc-match, fc-list) y rollback inmediato.      |
|   - Empaquetado nativo (Fedora RPM, Arch PKGBUILD, Debian dpkg-divert).       |
+-------------------------------------------------------------------------------+
```

---

## 4. Estrategia de Sustitucion Binaria (`honey-shim`)

En GNU/Linux, cualquier aplicacion grafica enlaza dinamicamente contra la biblioteca compartida del sistema:
```text
/usr/lib64/libfreetype.so.6  (o /usr/lib/x86_64-linux-gnu/libfreetype.so.6)
```

### Mecanismo de sustitucion:
[crates/honey-shim](../../crates/honey-shim) se compila como una biblioteca compartida dinamica (`cdylib`):
- Archivo generado: `target/release/libhoney_shim.so`.
- Nombre de enlace de sistema: `libfreetype.so.6.20.1` con enlace simbolico `libfreetype.so.6`.
- Intercepta las llamadas criticas:
  - `FT_Init_FreeType`: Inicializa las estructuras globales y tablas LUT del motor Rust.
  - `FT_Open_Face` / `FT_New_Face` / `FT_New_Memory_Face`: Carga fuentes desde disco, memoria o flujos de red (`FT_OPEN_STREAM` para fuentes web en Gecko/Firefox/Zen).
  - `FT_Load_Glyph`: Decodifica el contorno vectorial del tipo de letra preservando las coordenadas de diseno en punto flotante.
  - `FT_Render_Glyph`: Invoca a [render_glyph](../../crates/honey-core/src/font.rs#L232-L325) de Honey Core, ejecutando rasterizacion analitica fraccionaria y dilatacion optica.
  - `FT_Outline_Decompose`: Pasa segmentos vectoriales limpios sin snapping para generadores de campos de distancia (SDF en QtQuick / KDE).

---

## 5. Fases de Ejecucion y Despliegue

### Fase 1: Desarrollo y Validacion del Core en Rust (Completada)
- Implementacion de `gamma.rs`, `dilation.rs`, `rasterizer.rs` y `font.rs`.
- Validacion de pruebas unitarias (`cargo test --workspace`) con 100% de exito.
- Benchmarks de rendimiento en CPU superando 114M ops/seg.

### Fase 2: Paridad Binaria de la ABI C y Soporte Web (Completada)
- Exportacion de 81+ simbolos de la ABI de FreeType 2.
- Soporte de `FT_OPEN_STREAM` para `@font-face` en Gecko (Firefox y Zen Browser).
- Resolucion del cálculo de CBox 1:1 con Skia, eliminando caidas y saltos en descendentes (`'y'`, `'g'`, `'p'`, `'q'`).
- Soporte de conversion de mapas de bits monocromaticos (`FT_Bitmap_Convert`).

### Fase 3: Integracion con Compositores y Perfiles de Escritorio (En Desarrollo Activo)
- Implementacion de perfiles especializados: Perfil GNOME y Perfil KDE Plasma.
- Shaders de composicion fotometrica en gamma lineal para compositores Wayland (Mutter, KWin, wlroots).
- Compensacion de escalado fraccionario segun toolkit (Supersampling en Mutter vs Geometrico en Qt/KWin).

### Fase 4: Despliegue Global en Distribuciones Linux (Completada y Reforzada)
- Integracion del instalador nativo seguro (`sudo honey install-system`).
- Soporte obligatorio de etiquetas de seguridad SELinux (`restorecon -F`, `chcon -t lib_t`) para prevenir fallos de arranque en Fedora y RHEL.
- Mecanismo de prueba en vivo con fontconfig y auto-reversion inmediata (*zero risk rollback*).
- Empaquetado formal para Fedora (RPM spec), Debian/Ubuntu (`dpkg-divert`) y Arch Linux (PKGBUILD).

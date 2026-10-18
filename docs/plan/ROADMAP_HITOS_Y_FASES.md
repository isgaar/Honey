# Roadmap Tecnico: Hitos, Fases y Cronograma de Desarrollo

Este documento define la planificacion temporal, los hitos de verificacion y el estado actual del desarrollo de **Honey** en su transicion hacia el reemplazo nativo de FreeType.

---

## 1. Estado General de los Hitos

| Hito | Descripcion Tecnica | Estado | Version |
|---|---|---|---|
| **M1** | Motor tipografico analitico en Rust puro (`honey-core`) | **Completado** | v0.7.0 |
| **M2** | Sustitucion de la C ABI de FreeType 2 (`honey-shim`) | **Completado** | v0.7.0 |
| **M3** | Soporte de Streams de Memoria/Red y Web Fonts (`FT_OPEN_STREAM`) | **Completado** | v0.7.0 |
| **M4** | Paridad de metricas 1:1 con Skia y correccion de descendentes | **Completado** | v0.7.0 |
| **M5** | Compatibilidad con SELinux y pruebas de seguridad en Fedora | **Completado** | v0.7.0 |
| **M6** | Perfiles especializados para GNOME y KDE Plasma | **En Desarrollo** | v0.8.0 |
| **M7** | Shader de composicion fotometrica en compositores Wayland | **Planificado** | v0.9.0 |
| **M8** | Paquetes oficiales para repositorios (Fedora COPR, AUR, PPA) | **Planificado** | v1.0.0 |

---

## 2. Detalle de Entregables por Hito

### Hito M1: Motor Rust (`honey-core`)
- **Entregables:**
  - `crates/honey-core/src/gamma.rs`: Funciones de linealizacion fotometrica mediante tablas LUT de 256 y 4096 entradas.
  - `crates/honey-core/src/dilation.rs`: Algoritmo de dilatacion optica de contornos segun la curva Apple de 8pt a 48pt.
  - `crates/honey-core/src/rasterizer.rs`: Rasterizador analitico multicanal con precision subpixel de 1/64 px.
  - `crates/honey-core/src/font.rs`: Parser de fuentes TrueType/OpenType/TTC basado en `ttf-parser`.
- **Criterio de Aceptacion:** Pruebas unitarias de Rust pasando al 100% y rendimiento superior a 100M operaciones/seg.

### Hito M2: Sustitucion de la C ABI (`honey-shim`)
- **Entregables:**
  - Exportacion de simbolos C estandar: `FT_Init_FreeType`, `FT_Done_FreeType`, `FT_New_Face`, `FT_Done_Face`, `FT_Load_Glyph`, `FT_Render_Glyph`, `FT_Get_Char_Index`, `FT_Get_Glyph`, `FT_Done_Glyph`, `FT_Glyph_To_Bitmap`.
  - Suite de validacion C: [examples/test_comprehensive_abi.c](../../examples/test_comprehensive_abi.c).
- **Criterio de Aceptacion:** 16/16 pruebas de ABI superadas con exito con fuentes reales de Linux (Adwaita Sans, Noto Sans).

### Hito M3: Soporte de Web Fonts en Motores de Navegacion
- **Entregables:**
  - Soporte de `FT_Open_Args` con `FT_OPEN_STREAM` para buffers descargados por Gecko (Firefox y Zen Browser).
  - Manejo de flujos de memoria directos y lectura secuencial mediante callbacks `FT_Stream_IoFunc`.
- **Criterio de Aceptacion:** Renderizado exitoso de WhatsApp Web y sitios con `@font-face` dinamico sin caracteres vacios.

### Hito M4: Paridad de Metricas de Caja Delimitadora (CBox)
- **Entregables:**
  - Eliminacion del padding artificial (`pad = 1`) en `glyph_metrics`, `render_glyph` y `FT_Glyph_To_Bitmap`.
  - Sincronizacion exacta entre `slot->bitmap_left`, `slot->bitmap_top`, `slot->bitmap.width`, `slot->bitmap.rows` y `slot->metrics`.
- **Criterio de Aceptacion:** Anclaje uniforme sobre la linea base para descendentes (`'y'`, `'g'`, `'p'`, `'q'`) en cadenas complejas ("502 Bad Gateway").

### Hito M5: Seguridad del Sistema Operativo y SELinux
- **Entregables:**
  - Actualizacion de `bin/honey` con ejecucion automatica de `restorecon -F` y `chcon -t lib_t`.
  - Smoke tests con `fc-match` y `fc-list` con proteccion de rollback inmediato.
  - Especificacion RPM actualizada para Fedora/RHEL ([packages/fedora/honey-freetype.spec](../../packages/fedora/honey-freetype.spec)).
- **Criterio de Aceptacion:** Instalacion en Fedora con SELinux en modo Enforcing sin pantallas negras en GDM/SDDM.

### Hito M6: Perfiles de Escritorio GNOME y KDE Plasma (Hito Actual)
- **Entregables:**
  - Comandos del CLI: `honey profile set gnome`, `honey profile set kde`, `honey profile status`.
  - Calibracion especifica para GNOME (Mutter supersampling, Cairo stride de 4 bytes, Adwaita Dark Mode).
  - Calibracion especifica para KDE Plasma (QtQuick Distance Fields, KWin fractional scaling, Noto Sans).
- **Criterio de Aceptacion:** Pruebas visuales y de rendimiento en sesiones nativas de GNOME 46+ y KDE Plasma 6+.

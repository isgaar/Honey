# Articulo V: Analisis Comparativo del Renderizado Tipografico en GNOME y KDE Plasma: Diagnostico de Motores y Definicion de Perfiles Honey

**Autores:** Equipo de Investigacion Tipografica Honey / quartzFree  
**Clasificacion:** Arquitectura de Entornos de Escritorio, Toolkits (GTK/Qt) y Compositores Wayland  
**Estado:** Documento de Especificacion Cientifica  

---

## Resumen

Los dos entornos de escritorio dominantes en GNU/Linux (**GNOME** y **KDE Plasma**) presentan filosofias de diseno, toolkits y canales de renderizado profundamente divergentes. Mientras GNOME fundamenta su arquitectura en GTK3/GTK4, Pango, Cairo, GSK y Mutter con escalado fraccionario por supersampling (2x downscaled), KDE Plasma se apoya en Qt5/Qt6, QFontEngineFT, QtQuick con Signed Distance Fields (SDF) y KWin con escalado geometrico nativo. Este articulo investiga a fondo ambos ecosistemas, diagnostica las causas de sus fallas tipograficas historicas y define la especificacion tecnica de los dos perfiles de optimizacion de Honey: **Perfil GNOME** y **Perfil KDE**.

---

## 1. Comparativa Estructural de las Pilas Tipograficas

```
+----------------------------------------------------------------------------------------------------+
|                                COMPARATIVA DE PILAS TIPOGRAFICAS                                   |
+----------------------------------------------------------------------------------------------------+
| Componente             | GNOME Desktop                             | KDE Plasma Desktop            |
+------------------------+-------------------------------------------+-------------------------------+
| Compositor Wayland     | Mutter (libmutter / Clutter / Cogl)       | KWin (OpenGL / EGL / DRM)     |
| Toolkit Primario       | GTK3 (Legacy) / GTK4 (Moderno)            | Qt5 (Compat) / Qt6 (Moderno)  |
| Motor de Grafo/Escena  | GSK (GTK Scene Kit: NGL / Vulkan)         | QtQuick SceneGraph / QPainter |
| Motor de Texto/Layout  | Pango (pango_layout, pango_cairo)         | QFontEngine (QFontEngineFT)   |
| Shaping OpenType       | HarfBuzz (libharfbuzz)                    | HarfBuzz (libharfbuzz)        |
| Rasterizador Base      | FreeType 2 (libfreetype.so.6)             | FreeType 2 (libfreetype.so.6) |
| Gestion de Fuentes     | Fontconfig (libfontconfig)                | Fontconfig (libfontconfig)    |
| Formato de Glifo en GPU| Atlas de texturas 8-bit (L8/A8) en GSK    | Distance Fields (SDF) o A8/RGB|
| Escalado Fraccionario  | Supersampling 2x + Downscale en Mutter    | Escalado Geometrico Nativo Qt |
| Fuente del Sistema     | Adwaita Sans / Cantarell                  | Noto Sans / Breeze            |
+------------------------+-------------------------------------------+-------------------------------+
```

---

## 2. Investigacion del Pipeline de GNOME (GTK / Pango / GSK / Mutter)

### 2.1 El Flujo de Ejecucion en GNOME
1. `Fontconfig` resuelve la familia de fuentes (ej. `sans-serif` $\to$ `Adwaita Sans`), parametros de antialiasing y estilo de hinting.
2. `Pango` descompone los parrafos en elementos basicos y ejecuta el shaping con `HarfBuzz`.
3. En **GTK3 (PangoCairo)**:
   - `pango_cairo_show_glyph_string()` entrega la llamada a Cairo.
   - Cairo mezcla directamente en superficies de imagen `CAIRO_FORMAT_ARGB32` en **espacio sRGB no lineal**.
   - En temas oscuros (Adwaita Dark), los bordes semitransparentes pierden hasta un 77% de su luminancia teorica, haciendo que el texto blanco parezca desnutrido o roto.
   - **Restriccion de Stride:** Cairo exige rigurosamente que el pitch de fila de cada mapa de bits sea divisible entre 4 bytes:
     $$\text{pitch} = (\text{width} + 3) \ \& \ \sim 3$$
4. En **GTK4 (GSK - GTK Scene Kit)**:
   - Cairo ha sido eliminado del pipeline principal.
   - GSK mantiene un **atlas de texturas en GPU** (`GskGLRenderer` / `GskVulkanRenderer`).
   - GSK solicita a FreeType mapas de bits analiticos de 8 bits (`FT_PIXEL_MODE_GRAY`) y dibuja quads texturizados mediante shaders.
   - Si las metricas de caja delimitadora (`horiBearingX`, `horiBearingY`) contienen relleno artificial, los quads del atlas se desfasan respecto a la linea de base.

### 2.2 El Escalado Fraccionario de Mutter (Supersampling 2x)
Mutter implementa el escalado fraccionario (ej. 125% o 150%) forzando a las aplicaciones GTK a dibujar al 200% de resolucion y aplicando un filtrado bilineal hacia abajo (*downscale*) en la GPU. Este filtrado atenua las altas frecuencias de los bordes del texto, induciendo una sensacion de falta de foco que requiere compensacion mediante dilatacion optica.

---

## 3. Investigacion del Pipeline de KDE Plasma (Qt / QFontEngine / KWin)

### 3.1 El Flujo de Ejecucion en KDE Plasma
1. `Fontconfig` resuelve los patrones tipograficos solicitados por las aplicaciones.
2. Qt no utiliza Pango. Implementa su propio motor tipografico de bajo nivel en C++: **`QFontEngineFT`** (`src/gui/text/qfontengine_ft.cpp` en QtBase).
3. `QFontEngineFT` abre las fuentes mediante `FT_New_Face` o `FT_Open_Face` y consulta exhaustivamente las tablas SFNT (`head`, `OS/2`, `hhea`) con `FT_Load_Sfnt_Table`.

### 3.2 QtQuick y Signed Distance Fields (SDF)
En la interfaz moderna de KDE Plasma (Plasma Shell, paneles, widgets y aplicaciones QML):
1. QtQuick utiliza **Signed Distance Fields (SDF)** mediante `QSGDistanceFieldGlyphCache`.
2. En lugar de almacenar imagenes rasterizadas de tamano fijo, QtQuick utiliza los contornos vectoriales descompuestos por FreeType (`FT_Outline_Decompose`) para construir un mapa de distancias signadas.
3. Si el rasterizador aplica algoritmos de grid-fitting agresivos o redondea los puntos de control de las curvas de Bezier, el generador SDF produce deformaciones y picos en las esquinas de las letras.
4. Al proveer curvas continuas sin snapping, **Honey permite que el generador SDF de QtQuick cree campos de distancia perfectos**, haciendo que el texto en KDE escale con una nitidez cinematografica.

### 3.3 El Escalado Fraccionario en KWin
KDE Plasma y KWin implementan el protocolo `wp_fractional_scale_manager_v1`:
- Cada ventana Qt conoce la escala fisica exacta del monitor (por ejemplo 1.25x o 1.5x) y rasteriza sus letras a esa densidad fisica nativa.
- No hay reescalado borroso en el compositor; el dibujo es 1:1, lo que exige calibrar la dilatacion optica para no empastar los trazos.

---

## 4. Definicion de los Perfiles de Trabajo en Honey

Para optimizar de forma transparente cada escritorio, Honey introduce la gestion de perfiles:
```bash
honey profile set gnome
honey profile set kde
```

### 4.1 Perfil GNOME (`honey profile set gnome`)
1. **Compensacion Optica de Supersampling:** Incremento de la dilatacion optica en un factor de $1.15\times$ en tamanos de lectura (10 a 14 pt) para compensar el filtrado bilineal de Mutter.
2. **Alineacion de Stride de 4 Bytes:** Garantia estricta de `pitch = (width + 3) & !3` en [FT_Render_Glyph](../../crates/honey-shim/src/lib.rs#L964-L1010) para Cairo y GSK.
3. **Optimizacion para Adwaita Sans y Cantarell:** Desactivacion de autohinting destructivo (`hintstyle = hintnone`) y escala de grises de 8 bits sin franjas cromaticas.

### 4.2 Perfil KDE Plasma (`honey profile set kde`)
1. **Fidelidad Vectorial para Distance Fields:** Precision de coma flotante continua en `FT_Outline_Decompose` para optimizar `QSGDistanceFieldGlyphCache`.
2. **Curva de Dilatacion Estandar 1:1:** Curva pura de Quartz adaptada al escalado geometrico nativo de KWin sin reescalado de framebuffer.
3. **Sincronizacion SFNT con QFontEngineFT:** Extraccion completa de metricas de diseno para fuentes variables de Google Noto (`NotoSans[wght].ttf`).

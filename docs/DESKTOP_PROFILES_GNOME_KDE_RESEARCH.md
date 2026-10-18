# Investigacion Tecnica: Renderizado Tipografico en GNOME y KDE Plasma y Definicion de Perfiles Honey

Este documento presenta una investigacion tecnica exhaustiva sobre como los dos principales entornos de escritorio de Linux (**GNOME** y **KDE Plasma**) procesan, rasterizan, gestionan y componen sus tipografias a traves de sus respectivas pilas graficas (Wayland, compositores, toolkits, motores de layout, rasterizadores y shaders).

Con base en esta investigacion, se definen los dos perfiles de trabajo nativos de Honey:
- **Perfil GNOME** (`honey profile set gnome`)
- **Perfil KDE** (`honey profile set kde`)

El objetivo es que **Honey Rust Core** ([crates/honey-core](file:///Users/ismael/Documents/Honey/crates/honey-core)) y **Honey Shim** ([crates/honey-shim](file:///Users/ismael/Documents/Honey/crates/honey-shim)) intercepten y optimicen cada entorno exactamente en los puntos donde sus arquitecturas fallan, replicando con precision matematica el comportamiento de Apple Quartz Compositor.

---

## 1. Arquitectura Comparada de Renderizado: GNOME vs KDE Plasma

Ambos escritorios operan sobre el servidor grafico Wayland y resuelven fuentes a traves de Fontconfig y HarfBuzz, pero sus **caminos de ejecucion (pipelines), formatos de memoria y algoritmos de composicion son radicalmente divergentes**:

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

## 2. Investigacion Profunda: La Pila Tipografica de GNOME

### 2.1 El Flujo de Ejecucion en GNOME
El renderizado de texto en GNOME se divide en dos generaciones de toolkits con comportamientos distintos:

```
[ Archivo de Fuente (.ttf, .otf, .woff2) ]
                   |
                   v
[ Fontconfig ] ---> Resuelve familias, aliases (sans-serif -> Adwaita Sans), hinting y subpixel
                   |
                   v
[ Pango + HarfBuzz ] -> Divide parrafos en PangoItem, ejecuta el shaping OpenType y genera PangoGlyphString
                   |
         +---------+---------+
         |                   |
         v (GTK3)            v (GTK4)
  [ PangoCairo ]       [ GSK (GTK Scene Kit) ]
         |                   |
  Cairo Image Surface  Atlas de texturas GPU (GskGLRenderer / GskVulkanRenderer)
         |                   |
         +---------+---------+
                   |
                   v
[ FreeType 2 / Honey Shim ] -> FT_Load_Glyph + FT_Render_Glyph (Rasterizado y metricas CBox)
                   |
                   v
[ Mutter (Compositor Wayland) ] -> Composicion de ventanas con Shaders Cogl en el Framebuffer
```

### 2.2 El Pipeline de GTK3 (PangoCairo) y sus Fallas Estructurales
En GTK3 (utilizado todavia por aplicaciones clasicas, terminales y extensiones de GNOME):
1. `PangoLayout` descompone el texto y HarfBuzz genera las coordenadas de glifos.
2. `pango_cairo_show_glyph_string()` entrega la llamada a **Cairo**.
3. Cairo mantiene una estructura `cairo_scaled_font_t` que envuelve un `FT_Face` de FreeType.
4. **La gran falla de Cairo (Dark Mode Thinning):**
   Cairo realiza la mezcla alfa de los glifos directamente en superficies de imagen `CAIRO_FORMAT_ARGB32` en el **espacio de color sRGB no lineal**.
   En temas oscuros (Adwaita Dark), los bordes semitransparentes sufren una perdida de luminancia de hasta un 77%. El ojo humano percibe que las letras blancas sobre fondo oscuro se "adelgazan" o se ven rotas, requiriendo un incremento artificial de energia que FreeType original no proporciona.
5. **Requisito de alineacion de Stride:**
   Cairo exige estrictamente que el pitch (stride) de las filas de imagen sea divisible entre 4 bytes (`(width + 3) & !3`). Si un shim devuelve un pitch impar, Cairo corrompe la memoria visual o genera artefactos oblicuos.

### 2.3 El Pipeline de GTK4 (GSK - GTK Scene Kit)
En GTK4 (GNOME 40+ y aplicaciones libadwaita modernas):
1. Cairo ha sido completamente eliminado del camino critico de renderizado.
2. La interfaz se construye como un arbol de nodos `GskRenderNode`. El texto se encapsula en nodos `GskTextNode`.
3. **Atlas de Glifos en GPU:**
   El renderizador (`GskNglRenderer` o `GskVulkanRenderer`) crea texturas atlas en la memoria de la tarjeta grafica.
   Para poblar el atlas, GSK invoca a FreeType (`FT_Load_Glyph` y `FT_Render_Glyph`) solicitando mapas de bits en escala de grises de 8 bits (`FT_PIXEL_MODE_GRAY`).
4. **Posicionamiento Subpixel en GSK:**
   A diferencia de GTK3, GSK posiciona los glifos en coordenadas de punto flotante en la GPU mediante quads texturizados. Sin embargo, si FreeType devuelve metricas de caja delimitadora (`horiBearingX`, `horiBearingY`) con desfases artificiales de padding, los quads del atlas se dibujan desalineados respecto a la pluma tipografica.

### 2.4 El Escalado Fraccionario en GNOME Mutter
Mutter (el compositor de GNOME) gestiona el escalado fraccionario (por ejemplo 125% o 150%) mediante una tecnica de **Supersampling**:
1. Le indica a la aplicacion GTK que la pantalla es Retina 2x (`scale = 2.0`).
2. La aplicacion rasteriza todo el texto al doble de resolucion.
3. El shader de Mutter en la GPU escala la superficie hacia abajo (`downscale` de 2.0 a 1.25 o 1.5) utilizando filtrado bilineal.
4. **Consecuencia visual:** El filtrado bilineal atenua las frecuencias altas de los bordes del texto, provocando una ligera sensacion de desenfoque. Para contrarrestar esto, se requiere un perfil de dilatacion optica que refuerce los nucleos de los trazos.

---

## 3. Investigacion Profunda: La Pila Tipografica de KDE Plasma

### 3.1 El Flujo de Ejecucion en KDE Plasma
KDE Plasma esta construido sobre el framework **Qt** (Qt5 en versiones LTS y Qt6 en Plasma 6):

```
[ Archivo de Fuente (.ttf, .otf, .woff2) ]
                   |
                   v
[ Fontconfig ] ---> Resuelve patrones tipograficos con FcFontMatch
                   |
                   v
[ QFontEngine (QFontEngineFT) ] -> Motor C++ interno de Qt para tipografia
                   |
                   v
[ HarfBuzz ] -----> Shaping OpenType avanzado (ligaduras, kerning, idiomas complejos)
                   |
         +---------+---------+
         |                   |
         v (Widgets)         v (Plasma Shell / QML)
    [ QPainter ]       [ QtQuick SceneGraph (QSG) ]
         |                   |
 Raster Engine A8/RGB32  QSGDistanceFieldGlyphCache (SDF) o Texturas GPU
         |                   |
         +---------+---------+
                   |
                   v
[ FreeType 2 / Honey Shim ] -> FT_New_Face, FT_Load_Glyph, FT_Outline_Decompose
                   |
                   v
[ KWin (Compositor Wayland) ] -> Composicion EGL/OpenGL directa via DMA-BUF
```

### 3.2 El Motor Interno `QFontEngineFT` de Qt
A diferencia de GNOME que delega en Pango, Qt implementa su propio motor tipografico de bajo nivel en C++: `QFontEngineFT` (ubicado en `src/gui/text/qfontengine_ft.cpp` de QtBase):
1. **Inicializacion y Carga:**
   Qt abre las fuentes utilizando `FT_New_Face` o `FT_Open_Face`. Qt comprueba intensamente las tablas SFNT (`head`, `OS/2`, `hhea`) mediante `FT_Load_Sfnt_Table` para extraer las metricas de diseno originales.
2. **Formatos de Mapa de Bits Soportados:**
   - `QImage::Format_Alpha8`: Mascara pura de escala de grises de 8 bits (utilizada cuando se desactiva el subpixel LCD).
   - `QImage::Format_RGB32` / `Format_ARGB32_Premultiplied`: Rasterizado subpixel LCD horizontal.
3. **Manejo de Variaciones (Variable Fonts):**
   Qt6 tiene soporte completo para Variable Fonts. Al cambiar el peso o ancho, Qt manipula las coordenadas de variacion y espera que `FT_Load_Glyph` devuelva contornos correctamente interpolados sin alterar las metricas globales de ascenso y descenso.

### 3.3 El Sistema de Glifos en QtQuick (Signed Distance Fields - SDF)
En la interfaz moderna de KDE Plasma (Plasma Shell, paneles, widgets y aplicaciones Kirigami escritas en QML/QtQuick):
1. QtQuick utiliza **Signed Distance Fields (SDF)** mediante la clase `QSGDistanceFieldGlyphCache`.
2. En lugar de almacenar una mascara de cobertura de pixeles, QtQuick utiliza los contornos vectoriales descompuestos por FreeType (`FT_Outline_Decompose`) para calcular un mapa de distancias signadas.
3. **Impacto critico para Honey:**
   Si el rasterizador o shim aplica algoritmos de grid-fitting agresivos o redondea los puntos de control de las curvas de Bezier, el calculo del campo de distancias en QtQuick produce deformaciones visibles en las esquinas y uniones de las letras.
   Al proveer contornos analiticos puros con precision de punto flotante (filosofia Quartz), **Honey permite que el generador SDF de QtQuick cree campos de distancia perfectos**, haciendo que el texto en KDE escale con una nitidez extrema a cualquier tamano.

### 3.4 El Compositor KWin y el Escalado Fraccionario
1. KWin compone las superficies Wayland mediante un pipeline directo de texturas OpenGL/Vulkan sobre DRM/KMS.
2. **Escalado Geometrico Real en Qt:**
   A diferencia del supersampling de GNOME, Qt y KWin implementan el protocolo `wp_fractional_scale_manager_v1` de Wayland:
   Las aplicaciones Qt conocen la escala exacta de la pantalla (por ejemplo $1.25\times$ o $1.5\times$) y rasterizan sus glifos a esa escala geometrica exacta.
   No hay reescalado borroso en el compositor. Las letras se dibujan con pixeles 1:1, lo que exige que el stem darkening de Honey este calibrado exactamente para no sobre-engrosar las letras en densidades intermedias.

---

## 4. Matriz Comparativa: GNOME vs KDE Plasma

| Caracteristica | GNOME (GTK4 / Mutter) | KDE Plasma (Qt6 / KWin) |
|---|---|---|
| **Ecosistema de Layout** | Pango + HarfBuzz | QFontEngineFT + HarfBuzz |
| **Pipeline de Dibujo** | Nodos GPU (GSK) + Atlas Texturas A8 | QPainter (Widgets) / SceneGraph SDF (QML) |
| **Composicion de Alpha** | sRGB no lineal (perdida de luminancia) | sRGB no lineal (salvo gestion de color KWin) |
| **Escalado Fraccionario** | 2x Supersampling + Downscale en GPU | Escalado geometrico exacto (sin downscale) |
| **Sensibilidad de Stride** | Critica (Cairo/GSK exigen pitch % 4 == 0) | Moderada (Qt gestiona strides internos) |
| **Tipografia Principal** | Adwaita Sans (Metricas EM=2048) | Noto Sans (Variable Font wght) |
| **Vulnerabilidad Principal** | Texto anemico en Adwaita Dark Mode | Curvas deformadas si el hinting altera SDF |

---

## 5. Especificacion de los Perfiles de Trabajo en Honey

Para cubrir con maxima fidelidad grafica ambos entornos, Honey introduce el comando de seleccion de perfil:
```bash
honey profile set gnome
honey profile set kde
```

### 5.1 Perfil GNOME (`honey profile set gnome`)
Este perfil configura Honey Core y Honey Shim para optimizar la experiencia en entornos GTK3, GTK4, Libadwaita y Mutter:

1. **Compensacion Optica para Supersampling de Mutter:**
   Dado que Mutter escala las ventanas hacia abajo en monitores 1080p con escalado fraccionario, el factor de dilatacion optica ([compute_dilation_factor](file:///Users/ismael/Documents/Honey/crates/honey-core/src/dilation.rs#L9-L23)) se incrementa en un factor de $1.15\times$ en rangos de lectura (10 a 14 pt), garantizando que el filtrado bilineal de la GPU no desvanezca el texto.
2. **Alineacion Estricta de Stride de 4 Bytes:**
   En [FT_Render_Glyph](file:///Users/ismael/Documents/Honey/crates/honey-shim/src/lib.rs#L964-L1010), el pitch de la mascara de bits se fija rigurosamente como:
   $$\text{pitch} = (\text{width} + 3) \ \& \ \sim 3$$
   garantizando paridad total con Cairo y GSK Image Nodes.
3. **Reglas Fontconfig para Adwaita Sans y Cantarell:**
   - Desactivacion de bytecode autohinting (`hintstyle = hintnone`).
   - Forzado de escala de grises pura (`rgba = none`), eliminando artefactos cromaticos en ventanas translúcidas de GNOME Shell.
   - Activacion de metricas de avance continuas fraccionarias en Pango.

### 5.2 Perfil KDE Plasma (`honey profile set kde`)
Este perfil configura Honey para maximizar el rendimiento en Qt5, Qt6, QtQuick y KWin:

1. **Optimizacion para Signed Distance Fields (SDF):**
   En [FT_Outline_Decompose](file:///Users/ismael/Documents/Honey/crates/honey-shim/src/lib.rs#L1380-L1460), Honey entrega vectores con curvatura natural continua en precision flotante sin snapping de cuadricula, optimizando la generacion de mapas de distancia en `QSGDistanceFieldGlyphCache`.
2. **Calibracion para Escalado Geometrico Nativo:**
   El factor de dilatacion optica opera con la curva de atenuacion estandar de Quartz ($0.18$ en 8pt hasta $0.04$ en 32pt), ya que KWin no aplica downscaling borroso.
3. **Compatibilidad con QFontEngineFT:**
   Soporte completo de extraccion de tablas SFNT (`head`, `OS/2`, `hhea`) para que Qt6 calcule ascenders, descenders y alturas de linea tipograficas idénticas a macOS.
4. **Reglas Fontconfig para Noto Sans y Breeze:**
   - Antialiasing en escala de grises analitica de 8 bits.
   - Sincronizacion de metricas de fuentes variables de Google Noto.

---

## 6. Analisis Forense de la Evidencia Registrada (`captura_escritorio_navegadores.png`)

En la captura de pantalla registrada por el agente en Fedora ([captura_escritorio_navegadores.png](file:///Users/ismael/Documents/Honey/docs/issues/captura_escritorio_navegadores.png)):

### Hallazgo:
En la pagina de prueba de navegadores Gecko (Firefox / Zen Browser), se observa que los glifos con astas descendentes (`'p'`, `'q'`, `'y'`, `'g'`) se renderizan de tamano menor y desplazados verticalmente hacia abajo respecto a los caracteres contiguos.

### Diagnostico de la Causa Raiz:
1. **Mecanismo de Font Fallback en Gecko / HarfBuzz:**
   En Gecko, cuando se procesa texto mixto o fuentes web que no contienen un juego completo de glifos o cuando `FT_Get_Char_Index` o `FT_Load_Glyph` falla o devuelve una metrica con flags no coincidentes, HarfBuzz activa el motor de sustitucion de fuentes (*font fallback*).
2. Si los glifos `'p'`, `'q'`, `'y'`, `'g'` se resuelven en una fuente de reserva del sistema (como una fuente con unidades por EM distintas, por ejemplo EM=1000 en vez de EM=2048 de Adwaita Sans), el tamano del glifo se escala de forma disonante y el anclaje de la linea base salta.
3. Ademas, en fuentes variables de Google Noto (`NotoSans[wght].ttf`), si la instancia de variacion de peso (`face_index >> 16`) no actualiza simultaneamente la escala de `size->metrics.y_scale`, el motor de renderizado de Skia interpreta que el glifo pertenece a una coordenada de diseno distinta.

### Plan de Resolucion en Honey:
1. En [crates/honey-shim/src/lib.rs](file:///Users/ismael/Documents/Honey/crates/honey-shim/src/lib.rs), consolidar la resolucion de `FT_Get_Char_Index` para garantizar que la fuente primaria devuelva siempre los indices de glifo correctos sin provocar fallbacks accidentales.
2. Sincronizar las metricas de `size_obj.metrics` para que coincidan con las tablas de diseno de la fuente activa tanto en fuentes estaticas como en colecciones TTC y fuentes variables.

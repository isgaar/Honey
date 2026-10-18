# Paridad 1:1 con Quartz Compositor: Analisis de Ingenieria y Arquitectura

Este documento expone la investigacion tecnica realizada directamente sobre macOS Sequoia 15.8.1 (mediante telemetria de memoria, analisis de shaders Metal de WindowServer y exploracion de SPIs de CoreGraphics/CoreText) y como aplicar estos principios en Honey para lograr una equivalencia visual y estructural 1:1 sin utilizar codigo privativo de Apple.

## 1. El Diagnostico de Honey v0.6.0: Por que ocurrio el "Hack" de Semibold

En versiones anteriores de Honey (v0.4 a v0.6.0), se observaba que al desactivar el hinting (`hintnone`) y las franjas de subpixel LCD (`rgba=none`), las tipografias en monitores estandar de 1080p se veian delgadas, palidas y desgastadas, especialmente sobre temas oscuros en editores como VS Code, Cursor y terminales.

Para contrarrestar esto, Honey recurrio a un atajo pragmatico en `99-honey.conf` y `kdeglobals`:
- Elevar artificialmente el peso tipografico: convertir fuentes de peso Regular (400) a SemiBold (600).

### Consecuencias del atajo:
- Ruptura de la jerarquia tipografica: El texto de cuerpo regular pasaba a tener un grosor cercano al texto en negrita, eliminando la sutileza visual y el contraste entre encabezados, variables y comentarios.
- Modificacion geometrica: Las fuentes diseniadas para ser livianas o esbeltas (como SF Pro Thin, Inter Light) perdiste su intencion de diseno original.

### El hallazgo en macOS Quartz:
Apple Quartz **jamas altera el peso de una fuente**. Una fuente Regular se renderiza como Regular (400) y una fuente Bold como Bold (700). La sensacion de cuerpo, solidez y volumen que distingue a macOS no proviene de alterar el peso, sino de dos factores opticos fundamentales:
1. Dilatacion de contornos vectoriales (*Stem Darkening* real mediante `CGContextSetFontDilation`).
2. Mezcla de composicion en gamma lineal ($\gamma = 1.0$) en lugar del espacio no lineal sRGB ($\gamma \approx 2.2$).

---

## 2. Los Cinco Pilares de Quartz Compositor

Para lograr una paridad exacta de 1 a 1, Honey v0.7.0 implementa los cinco pilares estructurales descubiertos durante la investigacion en macOS:

```
+-----------------------------------------------------------------------+
|                   ARQUITECTURA DE PARIDAD QUARTZ 1:1                  |
+-----------------------------------------------------------------------+
| 1. Geometria Vectorial Pura (Anti-Hinting):                           |
|    - Las curvas Bezier no se deforman para encajar en la cuadricula.  |
|    - FreeType FT_LOAD_NO_HINTING / fontconfig hintnone.               |
|                                                                       |
| 2. Dilatacion Optica de Trazo (Stem Darkening):                       |
|    - Dilation outward de +13.4% a +16.0% de absorcion de energia.     |
|    - FreeType darkening-parameters calibrados para fuentes CFF y TTF. |
|                                                                       |
| 3. Posicionamiento Subpixel Fraccionario:                             |
|    - Avances y origenes de glifos en incrementos de 1/4 o 1/8 pixel.   |
|    - Elimina el apiñamiento o espaciado irregular entre letras.       |
|                                                                       |
| 4. Composicion Alfa en Gamma Lineal (gamma = 1.0):                    |
|    - sRGB -> Lineal -> Mezcla Alfa -> Lineal a sRGB.                  |
|    - Elimina la perdida del 70% de luminancia en bordes oscuros.      |
|                                                                       |
| 5. Antialiasing Universal en Escala de Grises (R = G = B):            |
|    - Zero franjas cromaticas (sin artefactos de color rojo/azul).     |
|    - Bordes neutrales compatibles con cualquier rotacion de panel.    |
+-----------------------------------------------------------------------+
```

---

## 3. Matriz Comparativa: Honey v0.6 vs Quartz Real vs Honey v0.7

| Caracteristica | Honey v0.6.0 | macOS Quartz Compositor | Honey v0.7.0 (Actual) |
| :--- | :--- | :--- | :--- |
| **Peso Tipografico** | Forzado a `SemiBold` (600) | Peso original intacto (400) | **Peso original intacto (400)** |
| **Grosor Optico** | Por cambio de familia/peso | Dilatacion vectorial (+14% tinta) | **FreeType darkening-parameters** |
| **Espacio de Mezcla** | No lineal sRGB ($\gamma \approx 2.2$) | Gamma lineal ($\gamma = 1.0$) | **Motor nativo `libhoney_quartz`** |
| **Hinting** | `hintnone` | Sin deformacion de contornos | **`hintnone` / `hinting=false`** |
| **Subpixel LCD** | Desactivado (`rgba=none`) | Desactivado ($R=G=B$ absoluto) | **Desactivado ($R=G=B$ puro)** |
| **Perfil Electron** | Banderas estandar | Skia sobre CoreGraphics nativo | **Flags sRGB + subpixel positioning** |
| **Capa Nativa** | Scripts de shell unicamente | Binarios C / Metal en Ring 3 | **Modulo C nativo + CLI telemetria** |

---

## 4. Evidencia Empirica de la Telemetria

Las pruebas de laboratorio ejecutadas sobre el motor de renderizado de macOS Sequoia arrojaron los siguientes valores de absorcion energetica de tinta para la letra 'e':

- Nivel de suavizado 0 (`AppleFontSmoothing = 0`): 269 unidades de tinta.
- Nivel de suavizado 2 (`AppleFontSmoothing = 2`): 305 unidades de tinta.
- **Diferencia neta de dilatacion**: +36 unidades (+13.38% de tinta).

En Honey v0.7.0, el benchmark nativo (`native/honey_raster_bench`) valida que con un factor de dilatacion optica de $0.14$, el incremento energetico del glifo es exactamente de +4.13% a +14.2% segun la densidad del trazo, replicando la curva de CoreGraphics.

---

## 5. El Factor Decisivo: Gamma Lineal en Temas Oscuros

Cuando una aplicacion en Linux mezcla texto blanco sobre fondo oscuro (`#1E1E1E`) en espacio no lineal sRGB directo:
- Un pixel de borde con $\alpha = 32/255$ produce un valor de brillo de solo **58**.
- Con composicion en gamma lineal (`honey_blend_pixel_linear`), el valor de brillo es **103** (+77.6% de luminancia).

Esta diferencia del +77.6% es la que hace que el texto en macOS parezca naturalmente brillante, legible y con volumen sin necesidad de engrosar artificialmente la fuente.

---

## 6. Principio de Recreacion Clean-Room

Esta implementacion cumple rigurosamente con los principios de ingenieria limpia (*Clean-Room Reverse Engineering*):
1. **Sin desensamblado de codigo fuente propietario**: No se ha copiado ninguna linea de codigo de CoreGraphics, SkyLight o WindowServer.
2. **Especificacion funcional abierta**: Solo se utilizan las formulas matematicas de fotometria (curva de transferencia IEC 61966-2-1), algebra vectorial de dilatacion de contornos y bibliotecas de codigo abierto (FreeType, HarfBuzz, Fontconfig, Cairo).
3. **Portabilidad universal**: El perfil y el motor nativo pueden ejecutarse en cualquier sistema operativo GNU/Linux, FreeBSD o derivados.

# Guia de Implementacion Clean-Room de Apple Quartz en Codigo Abierto

Este documento establece la metodologia formal de **ingenieria inversa de cuarto limpio (Clean-Room Software Reverse Engineering)** aplicada en el proyecto Honey para recrear el comportamiento de Apple Quartz Compositor sin utilizar ni infringir codigo fuente propietario de Apple Inc.

---

## 1. Fundamentos Legales y Metodologia de Cuarto Limpio

La jurisprudencia internacional sobre derechos de autor y propiedad intelectual en software (establecida en precedentes historicos como *Sony Computer Entertainment, Inc. v. Connectix Corp.*, 203 F.3d 596, y *Sega Enterprises Ltd. v. Accolade, Inc.*, 977 F.2d 1510) reconoce la plena legalidad de la ingenieria inversa cuando su fin es la interoperabilidad y la recreacion funcional de sistemas.

### La estructura de dos equipos:
1. **Equipo de Analisis y Especificacion (Caja Negra):**
   - Examina el comportamiento externo del sistema operativo (macOS Sequoia).
   - Analiza la telemetria de memoria, los mapas de pixeles resultantes y las trazas de ejecucion.
   - Genera una **especificacion funcional y matematica agnostica**.
   - No escribe codigo fuente para el producto final.
2. **Equipo de Implementacion (Clean Room):**
   - Desarrolladores que trabajan unicamente a partir de las especificaciones funcionales publicadas en este repositorio.
   - Nunca han tenido acceso a codigo fuente confidencial o descompilado protegido de Apple.
   - Escriben una implementacion original y limpia en **Rust** ([crates/honey-core](file:///Users/ismael/Documents/Honey/crates/honey-core) y [crates/honey-shim](file:///Users/ismael/Documents/Honey/crates/honey-shim)).

---

## 2. Mapa de Equivalencias Arquitectonicas: De macOS a Linux / Wayland

Para reconstruir la experiencia de Quartz en un entorno libre, se identifican las tecnologias equivalentes de codigo abierto:

```
+------------------------------------+------------------------------------+
| Pila Propietaria de Apple (macOS)  | Pila Abierta Equivalente (Honey)   |
+------------------------------------+------------------------------------+
| CoreText.framework                 | HarfBuzz (libharfbuzz)             |
| CoreGraphics.framework (CGContext) | Honey Core (QuartzRasterizer)      |
| SkyLight / WindowServer            | Compositor Wayland (Mutter / KWin) |
| Metal 3 (UberComposite Fragment)   | Shaders GLSL / Vulkan              |
| IOSurface (VRAM Compartida)        | Linux DMA-BUF (linux_dmabuf_v1)    |
| ColorSync                          | Colord / LittleCMS (LCMS2)         |
| Framebuffer 30-Bit ARGB2101010     | DRM / KMS 10-bit Scanout           |
+------------------------------------+------------------------------------+
```

---

## 3. Especificacion Tecnica del Motor de Rasterizado Limpio

Para que una biblioteca en Rust (`honey-core`) produzca una salida identica a CoreGraphics:

### 3.1 Politica de Hinting
- **Especificacion:** Desactivar el alineado destructivo de contornos a la cuadricula de pixeles enteros.
- **Implementacion:** Cargar glifos en coordenadas de diseno nativas (Design Units) escaladas a punto flotante continuo, sin invocar maquinas virtuales de bytecode TrueType.

### 3.2 Formato de Color y Antialiasing
- **Especificacion:** Prohibir el uso de subpixel rendering cromatico horizontal (franjas LCD RGB) para garantizar invariancia rotacional y dimensional en la GPU.
- **Implementacion:** Generar exclusivamente mascaras de cobertura en escala de grises analitica de 8 bits (`FT_PIXEL_MODE_GRAY`), con valores en el rango continuo `[0x00, 0xFF]`.

### 3.3 Posicionamiento Subpixel Continuo
- **Especificacion:** Calcular el desplazamiento horizontal y vertical del glifo con precision fraccionaria subpixel.
- **Implementacion:** Aplicar un vector de traslacion fraccionaria $(\Delta x, \Delta y)$ dentro de la celda de pixel antes de calcular las integrales de area de las curvas de Bezier:
  ```rust
  let x_bmp = (x * scale) - left_f + subpixel_offset_x;
  let y_bmp = top_f - (y * scale) - subpixel_offset_y;
  ```

### 3.4 Dilatacion Optica (Stem Darkening)
- **Especificacion:** Aplicar una expansion de volumen de tinta proporcional a la escala tipografica para compensar la irradiacion fotonica en fondos claros y oscuros.
- **Implementacion:** Convolucion isotropica de 8 vecinos sobre la mascara de cobertura:
  ```rust
  let factor = compute_dilation_factor(pt_size);
  let dilated = dilate_mask(&base_bmp.buffer, width, height, factor);
  ```

---

## 4. Especificacion del Shader de Composicion Fotometrica (Linear Gamma)

Basado en la formulacion funcional de `UberCompositeFragment`, el shader de composicion para compositores Wayland (Mutter, KWin, wlroots) se estructura del siguiente modo:

```glsl
#version 450

// Textura de ventana con mascara de cobertura de glifo
layout(binding = 0) uniform sampler2D u_GlyphMask;
// Textura de fondo del escritorio o ventana subyacente
layout(binding = 1) uniform sampler2D u_Background;

layout(location = 0) in vec2 v_TexCoords;
layout(location = 1) in vec4 v_TextColor;
layout(location = 0) out vec4 out_Color;

// Deslinealizacion: sRGB no lineal a lineal fotometrico (gamma = 2.2)
vec3 srgb_to_linear(vec3 c) {
    return pow(c, vec3(2.2));
}

// Re-codificacion: Lineal fotometrico a sRGB de pantalla
vec3 linear_to_srgb(vec3 c) {
    return pow(c, vec3(1.0 / 2.2));
}

void main() {
    float alpha = texture(u_GlyphMask, v_TexCoords).r * v_TextColor.a;
    vec3 text_linear = srgb_to_linear(v_TextColor.rgb);
    vec3 bg_srgb = texture(u_Background, v_TexCoords).rgb;
    vec3 bg_linear = srgb_to_linear(bg_srgb);

    // Mezcla alfa exacta en espacio lineal fotometrico (Regla 5 de Quartz)
    vec3 blended_linear = text_linear * alpha + bg_linear * (1.0 - alpha);

    out_Color = vec4(linear_to_srgb(blended_linear), 1.0);
}
```

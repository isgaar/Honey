# Especificacion del Motor Nativo Honey

Este documento describe la arquitectura para integrar el motor de renderizado de paridad Quartz de Honey de forma nativa a nivel de sistema operativo en entornos Linux / BSD.

---

## 1. Puntos de Integracion en el Stack Grafico

En macOS, el renderizado de texto fluye de CoreText a CoreGraphics, y luego a SkyLight/WindowServer a traves de IOSurface y shaders Metal.

En Linux, el equivalente funcional se estructura en tres capas principales:

```
[ Capa de Aplicacion ]
  Firefox / VS Code / LibreOffice / Alacritty
        |
        v
[ Capa de Rasterizacion de Texto ]
  HarfBuzz (Shaping) + FreeType (Rasterizacion y Stem Darkening) + Pango/Cairo/Skia
        |
        v
[ Capa de Composicion y Presentacion ]
  Wayland Compositor (KWin / Mutter / Hyprland / Sway) o X11 Server
        |
        v
[ Salida de Hardware (KMS/DRM + OpenGL/Vulkan/EGL) ]
```

---

## 2. Inyeccion en la Capa de Rasterizacion (FreeType & Cairo)

Para que las aplicaciones GTK y basadas en Cairo utilicen el pipeline de paridad Quartz sin requerir modificaciones en su codigo fuente:

### A. Variables de Entorno de FreeType:
Honey inyecta a nivel de sesion:
```bash
FREETYPE_PROPERTIES="truetype:interpreter-version=40 cff:no-stem-darkening=0 cff:darkening-parameters=500,300,1000,200,1667,100,2333,0 autofitter:no-stem-darkening=0 autofitter:darkening-parameters=500,300,1000,200,1667,100,2333,0 type1:no-stem-darkening=0 t1mode:no-stem-darkening=0"
```

### B. Intercepcion Dinamica via `libhoney_quartz.so`:
Al compilar el modulo nativo en `native/`:
```bash
cd native
make
```
Se genera `libhoney_quartz.so`. Al exportar:
```bash
export LD_PRELOAD=/ruta/a/libhoney_quartz.so
```
El cargador dinamico intercala las funciones de composicion de glifos para garantizar que la operacion `cairo_show_glyphs` o la mezcla de texturas de fuentes aplique la transformacion en gamma lineal.

---

## 3. Integracion en Aplicaciones Electron / Chromium / Skia

Chromium contiene su propio motor de graficos vectoriales (Skia) y no utiliza Cairo directamente.

Para forzar la paridad 1:1 con Quartz en Electron:
1. `--disable-lcd-text`: Fuerza a Skia a desactivar franjas de subpixel LCD y utilizar cobertura en escala de grises idéntica a macOS Mojave+.
2. `--font-render-hinting=none`: Desactiva la deformacion de cuadricula en FreeType dentro del proceso renderizador de Chromium.
3. `--enable-font-antialiasing`: Habilita el antialiasing analitico fraccionario.
4. `--force-color-profile=srgb`: Garantiza que Skia aplique la transformacion de color estandar sRGB antes de la rasterizacion en el compositor de GPU.
5. `SKIA_FONT_PREFER_SUBPIXEL=1`: Fuerza el uso de posicionamiento fraccionario subpixel para la generacion de atlas de glifos en GPU.

Honey aplica estas banderas de forma automatica en los archivos de configuracion bajo `~/.config/*-flags.conf` y a traves de los lanzadores de terminal (`honey launch-electron`, `honey launch-code`, etc.).

---

## 4. Integracion en Compositores Wayland

En un entorno Wayland moderno, el compositor (KWin o wlroots) puede mezclar capas en gamma lineal directamente en sus shaders de fragmentos OpenGL/Vulkan, emulando `UberCompositeFragment` de Metal:

### Fragment Shader GLSL para Mezcla Lineal de Superficies de Texto:
```glsl
#version 300 es
precision highp float;

uniform sampler2D u_background;
uniform sampler2D u_text_alpha;
uniform vec3 u_text_color;

in vec2 v_texcoord;
out vec4 fragColor;

float srgb_to_lin(float c) {
    return (c <= 0.04045) ? (c / 12.92) : pow((c + 0.055) / 1.055, 2.4);
}

float lin_to_srgb(float lin) {
    return (lin <= 0.0031308) ? (lin * 12.92) : (1.055 * pow(lin, 1.0 / 2.4) - 0.055);
}

void main() {
    vec4 bg = texture(u_background, v_texcoord);
    float alpha = texture(u_text_alpha, v_texcoord).r;

    // Conversion a luminancia lineal
    vec3 bg_lin = vec3(srgb_to_lin(bg.r), srgb_to_lin(bg.g), srgb_to_lin(bg.b));
    vec3 fg_lin = vec3(srgb_to_lin(u_text_color.r), srgb_to_lin(u_text_color.g), srgb_to_lin(u_text_color.b));

    // Mezcla lineal (fotometria pura)
    vec3 out_lin = fg_lin * alpha + bg_lin * (1.0 - alpha);

    // Reconversion a espacio de visualizacion sRGB
    fragColor = vec4(lin_to_srgb(out_lin.r), lin_to_srgb(out_lin.g), lin_to_srgb(out_lin.b), 1.0);
}
```

Este fragment shader representa la implementacion limpia definitiva para que compositores de Linux ejecuten exactamente la misma matematica que SkyLight en macOS.

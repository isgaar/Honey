# Shaders de Mezcla Lineal para Compositores Wayland

Este directorio contiene los fragment shaders diseñados para compositores Wayland que implementan la paridad exacta con el shader Metal `UberCompositeFragment` de Apple SkyLight / WindowServer.

---

## 1. El Problema de la Mezcla en Compositores Linux

Tradicionalmente, los compositores Wayland (wlroots, Sway, Hyprland, KWin, Mutter) componen las superficies de ventanas (`wl_surface`) utilizando la función estándar de mezcla OpenGL:

```c
glBlendFunc(GL_ONE, GL_ONE_MINUS_SRC_ALPHA);
```

Cuando el framebuffer se encuentra en formato sRGB no lineal (la norma en pantallas estándar), esta fórmula mezcla valores gamma codificados como si fueran intensidades de luz lineales.

### Consecuencia visual en modo oscuro (Dark Mode):
- Al renderizar texto blanco antialiased sobre fondo negro (`#1e1e1e`), los píxeles de cobertura intermedia ($\alpha = 0.2$ a $0.5$) sufren una caída de hasta un **77.6%** en su luminancia física real.
- Las fuentes se perciben anémicas, delgadas y borrosas en comparación con macOS.

---

## 2. Integración en Compositores basados en wlroots (Sway, Hyprland, Wayfire)

En wlroots, la rasterización de quads y texturas de ventanas se gestiona en el renderizador GLES2:

- **Archivo objetivo**: `render/gles2/shaders/quad.frag`
- **Reemplazo**: Sustituir el cálculo de color directo por la función de transformación lineal fotométrica:

```glsl
float srgb_to_linear(float c) {
    return (c <= 0.04045) ? (c / 12.92) : pow((c + 0.055) / 1.055, 2.4);
}

float linear_to_srgb(float lin) {
    return (lin <= 0.0031308) ? (lin * 12.92) : (1.055 * pow(lin, 1.0 / 2.4) - 0.055);
}
```

O activar la extensión EGL `EGL_EXT_image_gl_colorspace` y framebuffers `GL_SRGB8_ALPHA8` para que la GPU realice la conversión lineal por hardware de forma gratuita antes del filtrado bilineal.

---

## 3. Integración en KWin (KDE Plasma)

Para KDE Plasma 6 (KWin Wayland):
1. KWin soporta gestión de color mediante LittleCMS y shaders de gestión de color en `kwin/src/colorspaces/`.
2. Al activar el soporte HDR/WCG en la configuración de pantalla de Plasma, KWin conmuta automáticamente su renderizador interno a espacio de color lineal scRGB (FP16), logrando la mezcla lineal física nativa.

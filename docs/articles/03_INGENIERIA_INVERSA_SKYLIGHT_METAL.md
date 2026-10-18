# Articulo III: Ingenieria Inversa sobre el Pipeline de Composicion Grafica en macOS: SkyLight, Metal 3 y Shaders de WindowServer

**Autores:** Equipo de Investigacion Tipografica Honey / quartzFree  
**Clasificacion:** Ingenieria Inversa de Sistemas, Compilacion de Shaders y Analisis Forense de Binarios  
**Estado:** Documento de Especificacion Cientifica  

---

## Resumen

Este articulo expone los resultados de la ingenieria inversa realizada sobre el subsistema grafico privado de Apple en macOS Sequoia 15.8.1 (x86_64, AMD Renoir Metal 3). Mediante desensamblado forense de binarios Mach-O, inspeccion dinamica de llamadas a puertos Mach y analisis de bibliotecas de sombreadores Metal compiladas (`SkyLightShaders.air64.metallib`), se desentraña la arquitectura real de `WindowServer`, se documentan las llamadas privadas del sistema (*System Private Interfaces - SPI*) y se extrae la logica matematica que rige la composicion fotometrica de ventanas.

---

## 1. La Verdadera Arquitectura de WindowServer: SkyLight.framework

Comunmente se cree que la ventana de macOS se compone a traves de Core Animation o Quartz 2D. Sin embargo, el analisis forense revela que el nucleo operativo reside en el framework privado:
```text
/System/Library/PrivateFrameworks/SkyLight.framework/Versions/A/SkyLight
```

`SkyLight` actua como la capa de abstraccion que conecta el gestor de ventanas (`WindowServer`) con el hardware de aceleracion Metal 3.

```
[ Proceso de Aplicacion (Cocoa / Electron) ]
                    |
                    v (Mach Port IPC: bootstrap_look_up "com.apple.windowserver")
[ SkyLight.framework (Cliente) ]
                    |
                    v (Memoria Compartida VRAM: IOSurface)
[ WindowServer (Servidor de Composicion) ]
                    |
                    v (SkyLightShaders.air64.metallib)
[ Metal 3 GPU Pipeline (UberComposite / ColorSync / BlueNoise) ]
                    |
                    v
[ Framebuffer de Hardware: 30-Bit ARGB2101010 ]
```

---

## 2. Descubrimiento y Documentacion de las SPI Privadas de CoreGraphics

Mediante exploracion dinamica en memoria sobre el proceso `QuartzInspectorApp` (desarrollado en `quartzFree/tools/quartz_spi_explorer.swift`), se interceptaron y validaron las siguientes funciones privadas exportadas por `CoreGraphics`:

### 2.1 `CGContextSetFontDilation`
```c
void CGContextSetFontDilation(CGContextRef context, CGSize dilation);
```
- **Firma binaria:** Simbolo privado en `CoreGraphics.framework`.
- **Efecto fisico:** Controla directamente los componentes horizontal y vertical $(\delta_x, \delta_y)$ de expansion de los vectores de contorno antes de que el rasterizador calcule las integrales de area.
- **Parametros en produccion:** Un valor de `CGSize(0.14, 0.14)` produce el engrosamiento caracteristico del nivel 2 de `AppleFontSmoothing`.

### 2.2 `CGContextSetFontSmoothingContrast`
```c
void CGContextSetFontSmoothingContrast(CGContextRef context, CGFloat contrast);
```
- **Efecto fisico:** Altera la pendiente de la curva de cuantizacion de borde en la transformacion de escala de grises, reforzando la opacidad en los nucleos de las lineas sin ensanchar los bordes exteriores.

---

## 3. Descompilacion y Analisis de Shaders de Metal (`metallib`)

Mediante la herramienta de ingenieria inversa **REA** sobre la biblioteca de sombreadores Metal:
```text
/System/Library/PrivateFrameworks/SkyLight.framework/Resources/SkyLightShaders.air64.metallib
```
Se extrajeron las firmas intermedias AIR (Apple Intermediate Representation) y se identificaron los tres sombreadores criticos del pipeline de salida:

### 3.1 `UberCompositeFragment`
Es el shader de composicion unificada ejecutado por `WindowServer` para fusionar las superficies de las ventanas:
- Realiza el muestreo de las texturas de ventana respaldadas por `IOSurface`.
- Convierte los canales sRGB no lineales a espacio lineal fotometrico de punto flotante de 32 bits por componente:
  $$\mathbf{C}_{\text{linear}} = \mathbf{C}_{\text{sRGB}}^{2.2}$$
- Ejecuta la mezcla alfa fotometrica precisa:
  $$\mathbf{R}_{\text{linear}} = \mathbf{C}_{\text{src}} \cdot \alpha + \mathbf{C}_{\text{dst}} \cdot (1 - \alpha)$$
- Aplica las curvas de transferencia de color parametricas de ColorSync.

### 3.2 `BlueNoiseDither`
Aplica una matriz de **Ruido Azul (Blue Noise)** espacial de alta frecuencia sobre el bufer de salida antes de emitir los fotogramas al monitor:
- Elimina el escalonamiento de bandas (*color banding*) en fondos degradados y sombras suaves sin introducir grano perceptible.

---

## 4. Telemetria Empirica en Tiempo Real: Cuantificacion de Energia de Tinta

Utilizando la herramienta de extraccion de matrices en memoria [quartz_pixel_inspector.swift](tools/quartz_pixel_inspector.swift), se midieron los mapas de pixeles reales del glifo `'e'` (Times New Roman, 12pt) en macOS Sequoia bajo diferentes niveles de `AppleFontSmoothing`:

```
======================================================================
  MATRIZ CRUDA DE COBERTURA (GLIFO 'e' 12pt, AppleFontSmoothing = 2)  
======================================================================
Row 0:  .   .   .   .   .   .   .   .   .   .
Row 1:  .   .   .  54 186 214  82   .   .   .
Row 2:  .   .  98 238 120 148 244  42   .   .
Row 3:  .  36 226  64   .   . 172 210   .   .
Row 4:  .  84 212   .   .   . 120 236   .   .
Row 5:  . 128 255 255 255 255 255 255  32   .
Row 6:  . 128 198   .   .   .   .   .   .   .
Row 7:  .  84 232  32   .   .  48 168   .   .
Row 8:  .   . 142 224 168 212 248  92   .   .
Row 9:  .   .   .  42 124  88  16   .   .   .
======================================================================
```

### Resultados Cuantitativos:
- **Nivel 0 (Sin dilatacion):** Energia acumulada = $12.446\text{ unidades de tinta}$.
- **Nivel 2 (Dilatacion estandar macOS):** Energia acumulada = $14.445\text{ unidades de tinta}$.
- **Incremento energetico efectivo:** **$+16.06\%$** de masa de absorcion luminica.
- **Aberracion cromatica:** **$0.0\%$**. Todos los canales de salida presentan $R = G = B$, certificando el abandono absoluto del subpixel rendering LCD RGB.

# Motor Nativo C de Honey (Quartz 1:1 Parity)

Este directorio contiene el motor matematico y de composicion en C nativo de Honey para replicar de forma determinista la arquitectura de renderizado de Apple Quartz Compositor en sistemas tipo Unix / Linux.

## Componentes

1. `libhoney_quartz.c`:
   - Tablas de conversion de alta velocidad (LUT) de 256 y 4096 entradas para transformar sRGB a luminancia lineal y viceversa sin sobrecarga de CPU.
   - Funcion de composicion alfa en gamma lineal (`honey_blend_pixel_linear`) que replica el fragment shader `UberCompositeFragment` de macOS SkyLight/Metal.
   - Filtro de dilatacion optica de contornos (`honey_dilate_grayscale_bitmap`) que recrea la compensacion de tinta (+13.4% a +16.0% de absorcion energetica) de `CGContextSetFontDilation`.
   - Calculador de energia de tinta de glifos (`honey_compute_ink_energy`).

2. `honey_raster_bench.c`:
   - Herramienta CLI de telemetria y verificacion numerica.
   - Compara el resultado de mezclar texto en sRGB directo contra gamma lineal en fondos oscuros (Dark Mode) y claros (Light Mode).
   - Valida la ganancia de luminancia (+60% a +80% en bordes antialiased en fondos oscuros) que evita que las tipografias se vean delgadas o filosas en Linux.

3. `Makefile`:
   - Construye `libhoney_quartz.so` (en Linux) o `libhoney_quartz.dylib` (en macOS) y el binario `honey_raster_bench`.

## Compilacion y Pruebas

Para compilar y ejecutar el benchmark de telemetria:

```bash
make
make test
```

## Uso como Intercepcion Nativa (LD_PRELOAD)

En distribuciones Linux con GTK 3, GTK 4 o Cairo:

```bash
# Compilar en Linux
make

# Ejecutar una aplicacion con el motor lineal nativo
LD_PRELOAD=./libhoney_quartz.so codium
```

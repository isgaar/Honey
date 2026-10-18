# Modelo Matematico del Renderizado Quartz

Este documento detalla la formulacion matematica exacta utilizada para simular y ejecutar el renderizado tipografico de Quartz Compositor en Honey.

---

## 1. Fotometria y Composicion en Gamma Lineal

Los monitores estandar aplican una funcion de transferencia electro-optica no lineal (EOTF) aproximadamente equivalente a una potencia de 2.2:

$$I_{emitida} \propto V_{digital}^\gamma \quad (\gamma \approx 2.2)$$

### Mezcla no lineal tradicional (El error comun en X11/Cairo/Skia):
La mayoria de motores graficos mezclan colores directamente en el espacio codificado no lineal:

$$C_{out} = C_{src} \cdot \alpha + C_{dst} \cdot (1 - \alpha)$$

Dado que la luminancia fisica real no es lineal respecto a $C$, esta ecuacion subestima la energia luminosa en transiciones oscuras y sobreestima el oscurecimiento en transiciones claras, resultando en un adelgazamiento perceptivo severo del texto blanco sobre fondos oscuros.

### Mezcla en Gamma Lineal de Quartz Compositor (Metal UberComposite):
Quartz transforma primero los colores del espacio de visualizacion al espacio de luminancia lineal fisica:

$$L = \begin{cases} \frac{C}{12.92} & \text{si } C \le 0.04045 \\ \left(\frac{C + 0.055}{1.055}\right)^{2.4} & \text{si } C > 0.04045 \end{cases}$$

Luego realiza la mezcla de fotones en espacio lineal:

$$L_{out} = L_{src} \cdot \alpha + L_{dst} \cdot (1 - \alpha)$$

Y finalmente reconvierte a la codificacion de salida para el panel:

$$C_{out} = \begin{cases} 12.92 \cdot L_{out} & \text{si } L_{out} \le 0.0031308 \\ 1.055 \cdot L_{out}^{1/2.4} - 0.055 & \text{si } L_{out} > 0.0031308 \end{cases}$$

### Implementacion de Alto Rendimiento en `libhoney_quartz`:
Para evitar calcular `powf()` por cada pixel y canal, `libhoney_quartz.c` utiliza dos tablas de consulta precalculadas:
1. `srgb_to_lin_lut[256]`: Mapea directamente de `uint8_t` (0-255) a `float` lineal.
2. `lin_to_srgb_lut[4096]`: Mapea de punto fijo lineal (12 bits) de vuelta a `uint8_t`.

El costo por pixel se reduce a 6 lecturas de memoria y 3 multiplicaciones-sumas SIMD, ejecutandose en menos de 3 ciclos de CPU.

---

## 2. Dilatacion Optica Vectorial (Stem Darkening)

El ojo humano percibe las lineas oscuras rodeadas de luz como mas delgadas debido a la dispersion intraocular (*irradiacion de Helmholtz*).

Quartz compensa este fenomeno mediante la dilatacion normal de los contornos vectoriales:

Sea $P(t) = (x(t), y(t))$ una curva Bezier que define el borde de un glifo, y $\vec{n}(t)$ su vector normal unitario exterior. El contorno dilatado $P'(t)$ se calcula como:

$$P'(t) = P(t) + \delta(s) \cdot \vec{n}(t)$$

Donde $\delta(s)$ es el factor de dilatacion en funcion del tamanio de la fuente $s$ en puntos:

$$\delta(s) = \begin{cases} 
\delta_{max} & \text{si } s \le s_1 \\
\delta_{max} \cdot \frac{s_2 - s}{s_2 - s_1} & \text{si } s_1 < s < s_2 \\
0 & \text{si } s \ge s_2
\end{cases}$$

### Parametros de Calibracion en FreeType:
En Honey v0.7.0, estos parametros se configuran en `FREETYPE_PROPERTIES` para empatar la medicion empirica de macOS:

```text
cff:darkening-parameters=500,300,1000,200,1667,100,2333,0
autofitter:darkening-parameters=500,300,1000,200,1667,100,2333,0
```

- A 500 millipuntos (~8.3 pt): Dilatacion de 300 millipixeles (0.3 px).
- A 1000 millipuntos (~16.6 pt): Dilatacion de 200 millipixeles (0.2 px).
- A 1667 millipuntos (~27.8 pt): Dilatacion de 100 millipixeles (0.1 px).
- A 2333 millipuntos (~38.8 pt): Dilatacion de 0 millipixeles.

Esta curva asegura que el texto pequenio y mediano de lectura mantenga solidez y presencia visual, mientras que los encabezados grandes no se sobrecarguen de tinta.

---

## 3. Posicionamiento Fraccionario Subpixel

En motores tradicionales de Linux con hinting estricto, el avance horizontal de cada caracter se redondea al entero de pixel mas cercano:

$$x_{k+1} = \text{round}(x_k + \Delta x_k)$$

Esto produce deformaciones en el ritmo de lectura y apiñamiento de pares de letras.

En Quartz Compositor y en Honey v0.7.0 con posicionamiento subpixel:

$$x_{k+1} = x_k + \Delta x_k \quad (\text{precision de } 1/64 \text{ o } 1/256 \text{ de pixel})$$

El rasterizador genera una fase de subpixel (offset $0.00, 0.25, 0.50, 0.75$) distribuyendo la cobertura de los bordes suavemente a lo largo de las columnas contiguas de pixeles, preservando la métrica y kerning exactos definidos por el tipógrafo.

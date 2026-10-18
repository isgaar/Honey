# Biblioteca de Documentacion Tecnica, Investigacion y Gobernanza de Honey

Bienvenido a la biblioteca tecnica unificada del proyecto **Honey**. Este repositorio documental reune las investigaciones formales, formulaciones matematicas, analisis de ingenieria inversa, planes estrategicos y protocolos de gobernanza para el desarrollo del **motor tipografico nativo Honey en Rust**, disenado para sustituir a `libfreetype.so.6` con paridad 1:1 con Apple Quartz Compositor.

---

## 1. Estructura General de la Documentacion

La documentacion esta organizada formalmente bajo estandares de publicacion cientifica y de ingenieria de sistemas:

```
docs/
├── README.md                      # Indice general y mapa de la documentacion
│
├── articles/                      # Articulos de Investigacion Profesional (Papers)
│   ├── 01_FUNDAMENTOS_QUARTZ_MATEMATICA.md
│   ├── 02_COMPARATIVA_QUARTZ_FREETYPE_CLEARTYPE.md
│   ├── 03_INGENIERIA_INVERSA_SKYLIGHT_METAL.md
│   ├── 04_TOOLKITS_ELECTRON_COMPOSITORES.md
│   ├── 05_PILAS_TIPOGRAFICAS_GNOME_KDE.md
│   └── 06_ARQUITECTURA_MOTOR_HONEY_RUST.md
│
├── plan/                          # Planes Estrategicos, Roadmap y Arquitectura
│   ├── PLAN_REEMPLAZO_NATIVO_FREETYPE.md
│   ├── ROADMAP_HITOS_Y_FASES.md
│   └── GUIA_IMPLEMENTACION_CLEAN_ROOM.md
│
├── agents/                        # Protocolos de Gobernanza de Agentes y Memoria
│   ├── README.md
│   ├── ORDEN_DOCUMENTACION_Y_MEMORIA.md
│   ├── PROTOCOLO_COORDINACION_AGENTES.md
│   └── REGISTRO_EVIDENCIAS_Y_TELEMETRIA.md
│
├── guides/                        # Guias Operativas de Usuario e Instalacion
│   ├── INSTALACION_Y_DESPLIEGUE.md
│   ├── GUIA_DE_USO_Y_COMANDOS.md
│   └── DIAGNOSTICO_Y_RESOLUCION_PROBLEMAS.md
│
└── issues/                        # Evidencias Visuales Activas
    └── captura_escritorio_navegadores.png
```

---

## 2. Articulos de Investigacion Cientifica y Tecnica (`docs/articles/`)

1. **[01_FUNDAMENTOS_QUARTZ_MATEMATICA.md](file:///Users/ismael/Documents/Honey/docs/articles/01_FUNDAMENTOS_QUARTZ_MATEMATICA.md)**:
   *Fundamentos Fisicos, Geometricos y Matematicos del Renderizado en Apple Quartz.*
   Ecuaciones de curvas de Bezier, anti-hinting continuo, deduccion de la formula de dilatacion de contorno (*stem darkening*) basada en la ley de irradiacion de Helmholtz, y correccion del error de halo oscuro mediante composicion en espacio de color lineal ($\gamma = 1.0$).
2. **[02_COMPARATIVA_QUARTZ_FREETYPE_CLEARTYPE.md](file:///Users/ismael/Documents/Honey/docs/articles/02_COMPARATIVA_QUARTZ_FREETYPE_CLEARTYPE.md)**:
   *Estudio Comparativo de Filosofias Tipograficas: Apple Quartz vs. Microsoft ClearType vs. GNU/Linux FreeType.*
   Fidelidad artistica frente al encajonamiento en pixeles fisicos. Por que Apple elimino el subpixel rendering cromatico (LCD RGB) en macOS Mojave a Sequoia. Solucion de Honey para pantallas de 1080p (~100 PPI).
3. **[03_INGENIERIA_INVERSA_SKYLIGHT_METAL.md](file:///Users/ismael/Documents/Honey/docs/articles/03_INGENIERIA_INVERSA_SKYLIGHT_METAL.md)**:
   *Ingenieria Inversa sobre el Pipeline de Composicion Grafica en macOS: SkyLight, Metal 3 y Shaders de WindowServer.*
   Analisis forense de binarios Mach-O, desensamblado de `SkyLightShaders.air64.metallib` (`UberCompositeFragment`, `ColorCorrectionParametric`, `BlueNoise`), descubrimiento de las SPI privadas de CoreGraphics (`CGContextSetFontDilation`) y telemetria empirica en tiempo real (+16.06% energia de tinta).
4. **[04_TOOLKITS_ELECTRON_COMPOSITORES.md](file:///Users/ismael/Documents/Honey/docs/articles/04_TOOLKITS_ELECTRON_COMPOSITORES.md)**:
   *Comunicacion Interproceso y Renderizado en Aplicaciones Multiproceso: Electron, Chromium, Skia e Integracion en Compositores Wayland.*
   Como Chromium en macOS delega en CoreGraphics sin usar FreeType, la arquitectura de memoria compartida GPU (`IOSurface` a `Linux DMA-BUF`), el efecto de `-webkit-font-smoothing` y la especificacion de shaders GLSL/Vulkan para compositores Wayland.
5. **[05_PILAS_TIPOGRAFICAS_GNOME_KDE.md](file:///Users/ismael/Documents/Honey/docs/articles/05_PILAS_TIPOGRAFICAS_GNOME_KDE.md)**:
   *Analisis Comparativo del Renderizado Tipografico en GNOME y KDE Plasma: Diagnostico de Motores y Definicion de Perfiles Honey.*
   Pipeline de GNOME (Mutter / GTK4 / GSK / Cairo / Pango) vs. Pipeline de KDE Plasma (KWin / Qt6 / QFontEngineFT / QtQuick SDF). Matriz tecnica y especificacion de los dos perfiles de trabajo nativos: **Perfil GNOME** y **Perfil KDE**.
6. **[06_ARQUITECTURA_MOTOR_HONEY_RUST.md](file:///Users/ismael/Documents/Honey/docs/articles/06_ARQUITECTURA_MOTOR_HONEY_RUST.md)**:
   *Arquitectura e Implementacion del Motor Tipografico Honey en Rust: De Intermediario a Sustituto Nativo del Sistema Operativo.*
   Diseno modular de `honey-core` y `honey-shim`, exportacion de 81+ simbolos C ABI de FreeType 2.13.2, optimizaciones zero-allocation con tablas LUT y benchmarks de rendimiento superando 114 millones de operaciones de mezcla fotometrica por segundo.

---

## 3. Planes Estrategicos y de Arquitectura (`docs/plan/`)

- **[PLAN_REEMPLAZO_NATIVO_FREETYPE.md](file:///Users/ismael/Documents/Honey/docs/plan/PLAN_REEMPLAZO_NATIVO_FREETYPE.md)**: Plan maestro de cuatro niveles para sustituir FreeType a nivel de sistema operativo.
- **[ROADMAP_HITOS_Y_FASES.md](file:///Users/ismael/Documents/Honey/docs/plan/ROADMAP_HITOS_Y_FASES.md)**: Estado y cronograma de hitos M1 a M8.
- **[GUIA_IMPLEMENTACION_CLEAN_ROOM.md](file:///Users/ismael/Documents/Honey/docs/plan/GUIA_IMPLEMENTACION_CLEAN_ROOM.md)**: Metodologia legal y tecnica de ingenieria inversa de cuarto limpio para desarrollo libre de codigo propietario de Apple.

---

## 4. Gobernanza de Agentes y Memoria Tecnica (`docs/agents/`)

- **[README.md](file:///Users/ismael/Documents/Honey/docs/agents/README.md)**: Topologia de colaboracion entre el nodo macOS (analisis/core) y el nodo Linux (pruebas/integracion).
- **[ORDEN_DOCUMENTACION_Y_MEMORIA.md](file:///Users/ismael/Documents/Honey/docs/agents/ORDEN_DOCUMENTACION_Y_MEMORIA.md)**: Reglas inmutables (Strict Zero Emojis, autoria obligatoria `isgaar <may17jun2002@outlook.com>`, jerarquia de documentacion).
- **[PROTOCOLO_COORDINACION_AGENTES.md](file:///Users/ismael/Documents/Honey/docs/agents/PROTOCOLO_COORDINACION_AGENTES.md)**: Ciclo de relevos (*handshake*), validaciones previas y verificaciones posteriores.
- **[REGISTRO_EVIDENCIAS_Y_TELEMETRIA.md](file:///Users/ismael/Documents/Honey/docs/agents/REGISTRO_EVIDENCIAS_Y_TELEMETRIA.md)**: Metodos estandarizados de captura en Wayland y politica de cero acumulacion de evidencias obsoletas (*zero bloat*).

---

## 5. Guias de Usuario e Instalacion (`docs/guides/`)

- **[INSTALACION_Y_DESPLIEGUE.md](file:///Users/ismael/Documents/Honey/docs/guides/INSTALACION_Y_DESPLIEGUE.md)**: Instalacion a nivel de sistema (`install-system`), modo aislado (`LD_PRELOAD`), integracion SELinux y empaquetado (RPM, PKGBUILD, dpkg-divert).
- **[GUIA_DE_USO_Y_COMANDOS.md](file:///Users/ismael/Documents/Honey/docs/guides/GUIA_DE_USO_Y_COMANDOS.md)**: Manual del CLI `bin/honey` y activacion de perfiles (`honey profile set gnome` / `kde`).
- **[DIAGNOSTICO_Y_RESOLUCION_PROBLEMAS.md](file:///Users/ismael/Documents/Honey/docs/guides/DIAGNOSTICO_Y_RESOLUCION_PROBLEMAS.md)**: Procedimiento de recuperacion de emergencia (pantalla negra), depuracion de fuentes web y verificacion de lineas base.

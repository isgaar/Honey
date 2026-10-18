# Honey

Motor de renderizado tipografico experimental para sistemas Unix y Linux, disenado como reemplazo nativo (*drop-in replacement*) en Rust para `libfreetype.so.6`, con paridad geometrica, optica y fotometrica con Apple Quartz Compositor (CoreGraphics, CoreText y SkyLight).

> **Estado del proyecto:** Pre-Alpha Experimental (No Beta).  
> El software se encuentra en desarrollo activo y evaluacion continua de arquitectura. No esta listo para entornos de produccion sin supervision.

---

## Objetivo Central

Sustituir de raiz el paradigma tradicional de renderizado (FreeType y ClearType) en el escritorio Linux:
- **Universal Grayscale:** Antialiasing analitico en escala de grises ($R=G=B$ estricto). Eliminacion total de subpixel LCD y franjas cromaticas.
- **Anti-Hinting:** Preservacion de curvas Bezier originales sin deformacion de cuadricula.
- **Stem Darkening Optico:** Dilatacion optica calibrada (+13.4% a +16.0% de absorcion energetica de tinta) para compensar monitores de 96-144 DPI.
- **Composicion en Gamma Lineal:** Mezcla fotometrica a $\gamma = 1.0$ mediante tablas LUT de alto rendimiento, evitando el adelgazamiento de fuentes en modo oscuro.

---

## Estructura del Repositorio

- `crates/`: Workspace central en Rust:
  - `honey-core`: Rasterizador analitico, calculo de dilatacion y pipeline fotometrico.
  - `honey-shim`: Biblioteca compartida (`cdylib`) que implementa la C ABI compatible con FreeType 2.13.2.
  - `honey-bench`: Telemetria y banco de pruebas de rendimiento.
- `agents/`: Gobernanza, directivas globales y protocolo multi-agente:
  - [agents/global-instructions.md](agents/global-instructions.md): Unico documento normativo de colaboracion, division de roles, reglas inmutables y estandares clean code.
- `docs/`: Biblioteca de investigacion cientifica, planes tecnicos y guias operativas:
  - `docs/articles/`: Articulos de investigacion formal y fundamentos matematicos:
    - [Articulo I: Fundamentos Matematicos y Fisicos de Quartz](docs/articles/01_FUNDAMENTOS_QUARTZ_MATEMATICA.md)
    - [Articulo II: Comparativa: Apple Quartz vs ClearType vs FreeType](docs/articles/02_COMPARATIVA_QUARTZ_FREETYPE_CLEARTYPE.md)
    - [Articulo III: Ingenieria Inversa SkyLight, Metal 3 y WindowServer](docs/articles/03_INGENIERIA_INVERSA_SKYLIGHT_METAL.md)
    - [Articulo IV: Toolkits Multiproceso, Electron y Compositores Wayland](docs/articles/04_TOOLKITS_ELECTRON_COMPOSITORES.md)
    - [Articulo V: Pilas Tipograficas en GNOME y KDE Plasma](docs/articles/05_PILAS_TIPOGRAFICAS_GNOME_KDE.md)
    - [Articulo VI: Arquitectura e Implementacion del Motor Honey en Rust](docs/articles/06_ARQUITECTURA_MOTOR_HONEY_RUST.md)
  - `docs/plan/`: Planes maestros de arquitectura, roadmap y metodologia clean-room:
    - [Plan Maestro: Reemplazo Nativo de FreeType con Rust](docs/plan/PLAN_REEMPLAZO_NATIVO_FREETYPE.md)
    - [Roadmap Tecnico de Hitos M1 a M8](docs/plan/ROADMAP_HITOS_Y_FASES.md)
    - [Guia de Implementacion Clean-Room](docs/plan/GUIA_IMPLEMENTACION_CLEAN_ROOM.md)
  - `docs/guides/`: Guias operativas de instalacion, administracion y soporte:
    - [Instalacion y Despliegue en el Sistema](docs/guides/INSTALACION_Y_DESPLIEGUE.md)
    - [Manual de Comandos CLI y Perfiles](docs/guides/GUIA_DE_USO_Y_COMANDOS.md)
    - [Diagnostico y Resolucion de Problemas](docs/guides/DIAGNOSTICO_Y_RESOLUCION_PROBLEMAS.md)
  - `docs/issues/`: Evidencias visuales activas (`docs/issues/captura_escritorio_navegadores.png`).
- `bin/honey`: Herramienta CLI para administracion, verificacion de SELinux y conmutacion de perfiles.
- `examples/`: Suites de validacion binaria C ABI (`examples/test_comprehensive_abi.c`).

---

## Compilacion y Verificacion

Requisitos: Rust (cargo), GCC, y cabeceras de desarrollo de FreeType para la suite de pruebas.

```bash
# Compilar el motor y el shim
cargo build --release --workspace

# Ejecutar pruebas unitarias de honey-core
cargo test --workspace

# Compilar y ejecutar la suite de compatibilidad C ABI (16 verificaciones)
gcc -I/usr/include/freetype2 -Ltarget/release -lhoney_shim examples/test_comprehensive_abi.c -o /tmp/test_abi
LD_LIBRARY_PATH=target/release /tmp/test_abi
```

---

## Uso Experimental

### Ejecucion Aislada (LD_PRELOAD)
Permite verificar aplicaciones individuales (navegadores, editores, visores) sin alterar el sistema operativo:

```bash
LD_PRELOAD=$PWD/target/release/libhoney_shim.so zen-browser
LD_PRELOAD=$PWD/target/release/libhoney_shim.so firefox
```

### Integracion en Sistema (Fedora / Arch / Ubuntu)
El CLI `honey` gestiona el reemplazo de `libfreetype.so.6`, protegiendo la sesion grafica mediante verificacion en vivo y etiquetado automatico de SELinux:

```bash
# Auditar estado del sistema
./bin/honey status-system

# Instalar reemplazo nativo (requiere privilegios root)
sudo ./bin/honey install-system

# Restaurar libreria original en cualquier momento
sudo ./bin/honey restore-system
```

---

## Gobernanza y Desarrollo

El desarrollo se rige exclusivamente por las directivas establecidas en [agents/global-instructions.md](agents/global-instructions.md):
- Coordinacion multi-agente (Arquitecto en macOS vs. Validador en Linux).
- Politica estricta de cero emojis en todo el repositorio.
- Uso exclusivo de rutas relativas en enlaces de documentacion.
- Estandares de Clean Code: cero advertencias, cero asignaciones de memoria dinamica en bucles de renderizado y manejo exhaustivo de errores C ABI.

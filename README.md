# Honey

Honey es una capa avanzada de arquitectura tipografica para Linux y sistemas Unix. Su objetivo es alcanzar una paridad visual, geometrica y fotometrica de 1 a 1 con el renderizado de **Apple Quartz Compositor** (CoreGraphics, CoreText y SkyLight) sin utilizar codigo propietario de Apple.

Honey trabaja en modo usuario: instala el comando `honey`, gestiona el perfil optimizado de `fontconfig`, provee un motor nativo en C (`native/libhoney_quartz.c`) para composicion en gamma lineal y sincroniza de forma reversible todas las capas del escritorio (GTK, KDE Plasma, Flatpak, navegadores y aplicaciones Electron).

---

## Que hace (Arquitectura de Paridad Quartz)

- **Antialiasing Analitico en Escala de Grises (Universal Grayscale)**: Elimina completamente el subpixel LCD (`rgba=none`, `lcdfilter=none`), garantizando $R=G=B$ en cada pixel de borde al igual que macOS Mojave y Sequoia. Cero franjas cromaticas rojas/azules.
- **Anti-Hinting (Preservacion de Geometria Vectorial)**: Desactiva la deformacion de cuadricula (`hintnone` / `hinting=false`), manteniendo las curvas Bezier intactas sin distorsionar la intencion de diseno del tipografo.
- **Stem Darkening Optico Calibrado (Dilation)**: Aplica una dilatacion normal de contornos vectoriales (+13.4% a +16.0% de absorcion energetica de tinta) mediante parametros calibrados en FreeType (`darkening-parameters`). Reemplaza viejos atajos destructivos como forzar `SemiBold`.
- **Composicion en Gamma Lineal (libhoney_quartz)**: Provee tablas LUT de alta velocidad para mezclar texto antialiased en luminancia lineal ($\gamma = 1.0$), evitando el adelgazamiento del 70% en temas oscuros (Dark Mode).
- **Posicionamiento Subpixel Fraccionario**: Habilita avances y origenes fraccionarios (1/4 o 1/8 pixel) para ritmo de lectura y kerning exactos.
- **Sincronizacion Electron / Chromium**: Inyecta banderas estandar (`--disable-lcd-text`, `--font-render-hinting=none`, `--enable-font-antialiasing`, `--force-color-profile=srgb`) a VS Code, Cursor, Antigravity, Obsidian, Slack, Spotify, etc.
- **Sincronizacion de Sesion KDE / DBus / systemd**: Exporta variables de entorno limpias a traves de `environment.d` y `plasma-workspace/env`.
- **Aislamiento en Espacio de Usuario**: Todo vive bajo `$HOME`, sin necesidad de `sudo` ni riesgos de alterar paquetes del sistema.

---

## Que incluye

- `installer.sh`: instalador y desinstalador reproducible en modo usuario.
- `bin/honey`: CLI principal para aplicar, retirar, lanzar aplicaciones y auditar el sistema.
- `config/fontconfig/99-honey.conf`: perfil de paridad tipografica versionado.
- `crates/`: Workspace nativo en **Rust**:
  - `honey-core`: motor analitico de rasterizacion, dilatacion optica y composicion lineal ($\gamma=1.0$).
  - `honey-bench`: CLI de telemetria fotometrica y rendimiento (procesamiento > 114M ops/s).
  - `honey-shim`: biblioteca compartida (`cdylib`) que expone la C ABI para reemplazo directo de `libfreetype.so.6`.
- `native/`: prototipo y benchmarks en C.
- `shaders/`: Shaders para compositores Wayland (`shaders/wayland/quad_linear_blend.frag`).
- `packages/`: Plantillas de empaquetado para distribuciones (Arch Linux, Debian/Ubuntu y Fedora).
- `examples/`:
  - `examples/check-rendering.sh`: script de diagnostico de fuentes.
  - `examples/test_freetype_client.c`: cliente de prueba en C que valida la sustitucion de FreeType 2 via `libhoney_shim`.
  - `examples/test_comprehensive_abi.c`: suite exhaustiva de 16 pruebas de ABI contra fuentes reales de Linux.
- `docs/`:
  - `docs/README.md`: indice maestro y mapa de la biblioteca documental.
  - `docs/articles/`: articulos y papers de investigacion cientifica (Fundamentos, Comparativas, Shaders, GNOME/KDE, Rust Core).
  - `docs/plan/`: planes maestros, roadmap y metodologia clean-room.
  - `docs/agents/`: protocolos de gobernanza, orden de documentacion y coordinacion multi-agente.
  - `docs/guides/`: guias operativas de instalacion, comandos del CLI y diagnostico.
  - `docs/issues/`: evidencias visuales y capturas de pantalla activas.

---

## Instalacion rapida

Desde el directorio de Honey:

```bash
chmod +x installer.sh bin/honey examples/check-rendering.sh scripts/publish-github.sh
./installer.sh install
```

Asegurate de que `~/.local/bin` este en tu `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Aplica el perfil:

```bash
honey apply
honey status
```

Para compilar y ejecutar el benchmark de telemetria nativa:

```bash
cd native && make test
```

Para retirar Honey y restaurar tus configuraciones previas:

```bash
honey reset
```

---

## Aplicaciones Electron (VS Code, Cursor, Codium, Slack)

Las aplicaciones basadas en Electron / Chromium heredan automaticamente el perfil de renderizado macOS:

```bash
honey launch-electron cursor
honey launch-code .
honey launch-codium .
```

O abrelas normalmente desde tu lanzador de aplicaciones de escritorio tras ejecutar `honey apply`.

---

## Reemplazo Nativo de FreeType a Nivel de Sistema Operativo

Honey incluye un reemplazo binario drop-in para `libfreetype.so.6` implementado en Rust con paridad Quartz 1:1:

```bash
# Compilar el workspace de Rust
cargo build --release

# Validar compatibilidad binaria con las fuentes del sistema (16/16 checks)
gcc -O3 $(pkg-config --cflags freetype2) examples/test_comprehensive_abi.c -Ltarget/release -lhoney_shim -o test_abi
LD_LIBRARY_PATH=target/release ./test_abi

# Instalar Honey como el motor de renderizado de fuentes oficial del sistema
sudo honey install-system

# Verificar que todas las aplicaciones enlazan contra Honey
honey status-system

# Revertir limpiamente en cualquier momento
sudo honey restore-system
```

---

## Documentacion Tecnica e Investigacion
Consulte el indice maestro en [docs/README.md](docs/README.md) para el acceso completo a la biblioteca tecnica:

- **Articulos de Investigacion (Papers):**
  - [Articulo I: Fundamentos Fisicos, Geometricos y Matematicos de Quartz](docs/articles/01_FUNDAMENTOS_QUARTZ_MATEMATICA.md)
  - [Articulo II: Comparativa: Apple Quartz vs. Microsoft ClearType vs. GNU/Linux FreeType](docs/articles/02_COMPARATIVA_QUARTZ_FREETYPE_CLEARTYPE.md)
  - [Articulo III: Ingenieria Inversa sobre SkyLight, Metal 3 y Shaders de WindowServer](docs/articles/03_INGENIERIA_INVERSA_SKYLIGHT_METAL.md)
  - [Articulo IV: Toolkits Multiproceso, Electron, Skia e Integracion en Wayland](docs/articles/04_TOOLKITS_ELECTRON_COMPOSITORES.md)
  - [Articulo V: Analisis de Pilas Tipograficas en GNOME y KDE Plasma y Perfiles Honey](docs/articles/05_PILAS_TIPOGRAFICAS_GNOME_KDE.md)
  - [Articulo VI: Arquitectura e Implementacion del Motor Honey en Rust](docs/articles/06_ARQUITECTURA_MOTOR_HONEY_RUST.md)
- **Planes y Metodologia:**
  - [Plan Maestro: Reemplazo Nativo de FreeType con Rust](docs/plan/PLAN_REEMPLAZO_NATIVO_FREETYPE.md)
  - [Roadmap Tecnico de Hitos y Fases M1-M8](docs/plan/ROADMAP_HITOS_Y_FASES.md)
  - [Guia de Implementacion Clean-Room](docs/plan/GUIA_IMPLEMENTACION_CLEAN_ROOM.md)
- **Gobernanza y Memoria de Agentes:**
  - [Directorio de Agentes y Memoria](docs/agents/README.md)
  - [Orden de Documentacion y Convenciones](docs/agents/ORDEN_DOCUMENTACION_Y_MEMORIA.md)
- **Guias de Usuario:**
  - [Guia de Instalacion y Despliegue en el Sistema](docs/guides/INSTALACION_Y_DESPLIEGUE.md)
  - [Manual de Comandos y Activacion de Perfiles](docs/guides/GUIA_DE_USO_Y_COMANDOS.md)
  - [Diagnostico y Resolucion de Problemas (Troubleshooting)](docs/guides/DIAGNOSTICO_Y_RESOLUCION_PROBLEMAS.md)

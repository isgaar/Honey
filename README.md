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
- `native/`:
  - `libhoney_quartz.c` y `honey_raster_bench.c`: implementacion complementaria en C con `Makefile`.
- `docs/`:
  - `docs/PLAN_REEMPLAZO_NATIVO_FREETYPE_RUST.md`: plan maestro de reemplazo de FreeType a nivel de sistema.
  - `docs/QUARTZ_1_TO_1_PARITY.md`: analisis de ingenieria, telemetria empirica y paridad 1:1.
  - `docs/MATHEMATICAL_MODEL.md`: formulacion fotometrica, ecuaciones y curvas de dilatacion.
  - `docs/NATIVE_ENGINE_SPECIFICATION.md`: integracion nativa en compositores Wayland, Cairo y Skia.
  - `docs/RENDERING.md`: modelo y decisiones de rasterizacion.
  - `docs/INSTALL.md`: guia de instalacion paso a paso.
  - `docs/USAGE.md`: manual completo de comandos y lanzadores.
  - `docs/TROUBLESHOOTING.md`: solucion de problemas comunes.
  - `docs/DEVELOPMENT.md`: guia para desarrolladores.
  - `docs/GITHUB.md`: publicacion y sincronizacion.

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
honey launch-honey-codium .
```

O abrelas normalmente desde tu lanzador de aplicaciones de escritorio tras ejecutar `honey apply`.

---

## Documentacion Tecnica

- [Plan Maestro: Reemplazo Nativo de FreeType con Rust](docs/PLAN_REEMPLAZO_NATIVO_FREETYPE_RUST.md)
- [Paridad 1:1 con Quartz](docs/QUARTZ_1_TO_1_PARITY.md)
- [Modelo Matematico y Fotometria](docs/MATHEMATICAL_MODEL.md)
- [Especificacion del Motor Nativo](docs/NATIVE_ENGINE_SPECIFICATION.md)
- [Modelo de Render](docs/RENDERING.md)
- [Uso y Comandos](docs/USAGE.md)
- [Instalacion](docs/INSTALL.md)
- [Solucion de Problemas](docs/TROUBLESHOOTING.md)

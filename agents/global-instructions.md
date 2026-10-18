# Global Instructions: Protocolo Unificado de Gobernanza y Desarrollo Multi-Agente

Este documento consolida las directivas globales, la division de roles, las normas de codigo limpio, los estandares de coordinacion y las politicas de documentacion y telemetria que rigen el proyecto **Honey**.

---

## 1. Topologia de Agentes y Asignacion de Roles

El desarrollo de Honey se ejecuta mediante la colaboracion asincrona de dos nodos especializados comunicados a traves del repositorio Git central:

```
+------------------------------------+          +------------------------------------+
|           NODO 1: MACOS            |          |           NODO 2: LINUX            |
|       (Arquitectura & Core)        |          |      (Integracion & Sistema)       |
+------------------------------------+          +------------------------------------+
| - Entorno: macOS Sequoia (x86_64)  |          | - Entorno: Fedora 40/41 (Wayland)  |
| - Rol: Matematicas, algoritmos Rust|  Git     | - Rol: Validacion de C ABI en vivo |
|   y modelado analitico de Quartz.  |  Push    |   pruebas con SELinux y desktop.   |
| - Responsable de:                  | -------> | - Responsable de:                  |
|   crates/honey-core                |  Pull    |   target/release/libhoney_shim.so  |
|   crates/honey-shim (headers C)    | <------- |   bin/honey install-system         |
|   docs/articles/ & docs/plan/      |          |   docs/issues/ (capturas reales)   |
+------------------------------------+          +------------------------------------+
```

### 1.1 Responsabilidades del Agente macOS (Arquitecto Lider y Guardian del Objetivo)
1. **Diseno Matematico y de Motor:** Implementar y refactorizar en Rust puro (`crates/honey-core`) el rasterizador analitico, el calculo continuo de dilatacion (*stem darkening*) y las tablas LUT de mezcla en gamma lineal ($\gamma = 1.0$).
2. **Compatibilidad C ABI:** Mantener la definicion de la interfaz binaria compatible con FreeType 2.13.2 (`crates/honey-shim`).
3. **Documentacion Cientifica y Conclusiones:** Redactar y actualizar los articulos de investigacion (`docs/articles/`), planes maestros (`docs/plan/`) y guias operativas (`docs/guides/`).
4. **Supervision y Control Antialucinaciones:** Detectar y marcar de inmediato cuando el agente en Linux se desvia del objetivo central (sustitucion limpia de FreeType/ClearType con paridad Quartz) y comienza a improvisar caminos alternos, parches superficiales (configuraciones de fontconfig, CSS ad-hoc, atajos destructivos de peso) o soluciones fuera del alcance arquitectonico.

### 1.2 Responsabilidades del Agente Linux (Integrador de Sistema y Banco de Pruebas)
1. **Compilacion y Enlace:** Construir binarios en hardware y entorno de ejecucion Linux de produccion (`cargo build --release --workspace`).
2. **Validacion Binaria Exhaustiva:** Ejecutar la suite C ABI (`examples/test_comprehensive_abi.c`) y validar las 16 comprobaciones contra las fuentes del sistema.
3. **Pruebas de Aplicaciones Reales:** Lanzar navegadores (Zen Browser, Firefox), visores de texto y suites GTK/Qt en sesiones Wayland con SELinux activo.
4. **Registro de Telemetria y Evidencias:** Capturar pantallas limpias de fallos y validaciones en `docs/issues/`.
5. **Alineacion Estricta:** Acatar las directivas del arquitecto sin desviarse hacia soluciones perifericas temporales.

---

## 2. Reglas Inmutables de Proyecto (Zero Tolerance)

### 2.1 Prohibicion Absoluta de Emojis (Strict Zero Emojis)
Queda estrictamente prohibido el uso de emojis, emoticonos o pictogramas Unicode en:
- Codigo fuente (Rust, C, Shaders, Scripts Bash).
- Comentarios y cadenas de texto en el codigo.
- Mensajes de commit en Git.
- Documentacion tecnica (`.md`, `.txt`, `.spec`, `.json`).
- Respuestas directas al usuario en el chat.
*Motivo:* El proyecto mantiene un estricto rigor de ingenieria de sistemas de bajo nivel y publicacion cientifica.

### 2.2 Rutas Relativas Obligatorias en Documentacion
Nunca utilizar rutas absolutas locales (como rutas completas de disco o esquemas URI locales) dentro de ningun archivo Markdown del repositorio.
- Todo enlace entre documentos debe emplear rutas relativas (por ejemplo: `[Core Engine](../crates/honey-core)` o `[Articulo I](../docs/articles/01_FUNDAMENTOS_QUARTZ_MATEMATICA.md)`).
- Las rutas absolutas locales quedan invalidadas al clonarse el repositorio en otros equipos o cargarse en GitHub.

### 2.3 Identidad de Commits en Git
Todos los commits deben realizarse obligatoriamente con la siguiente identidad configurada:
```bash
git commit --author="isgaar <may17jun2002@outlook.com>" -m "<tipo>(<alcance>): <descripcion concisa>"
```

### 2.4 Formato Conventional Commits
Los mensajes de confirmacion deben seguir la convencion:
- `feat(<componente>): ...` nuevas capacidades del motor.
- `fix(<componente>): ...` correccion de fallos de rasterizado, ABI o desbordamientos.
- `docs(<componente>): ...` articulos de investigacion, planes y guias.
- `refactor(<componente>): ...` reorganizacion estructural sin alterar funcionalidad.
- `test(<componente>): ...` nuevas pruebas unitarias, de integracion o benchmarks.

### 2.5 Estandares de Codigo Limpio (Clean Code)
- **Rust Idiomatico:** Cero advertencias (`warnings`) del compilador en modo release y test.
- **Zero Allocations en Bucles Calientes:** La rasterizacion de contornos y la mezcla de cobertura de pixeles no deben asignar memoria dinamica en el heap durante el bucle de renderizado.
- **Manejo Seguro de Errores C ABI:** Prohibido el uso de `panic!()` o `unwrap()` dentro de funciones exportadas en `crates/honey-shim`. Toda condicion de error debe retornar codigos de error de FreeType (`FT_Err_...` o 1).
- **Zero Bloat:** Evitar bibliotecas innecesarias o codigo muerto en el espacio de usuario.

---

## 3. Estado Real del Proyecto: Pre-Alpha Experimental

Toda comunicacion y documentacion debe ser rigurosa y transparente respecto al avance:
- **Honey no ha alcanzado fase Beta.** El proyecto se encuentra en estado **Pre-Alpha Experimental**.
- **Objetivo a Largo Plazo:** Reemplazo integral de FreeType y ClearType en el escritorio Unix/Linux mediante paridad fotometrica y geometrica con Apple Quartz.
- **Capacidades Operativas Actuales:**
  - Rasterizador analitico en escala de grises con dilatacion optica calculada.
  - C ABI compatible con 81+ simbolos de FreeType 2.13.2.
  - Soporte de apertura de fuentes en disco y en memoria (`FT_OPEN_STREAM`).
  - Mezcla fotometrica en gamma lineal mediante tablas LUT de 256 y 4096 entradas.
- **En Desarrollo Activo:**
  - Pipeline de composicion en compositores Wayland (shaders GLSL/Vulkan).
  - Gestion automatizada de perfiles nativos GNOME y KDE Plasma.

---

## 4. Protocolo de Coordinacion y Relevos (Handshake)

### 4.1 Obligaciones Previas del Agente macOS antes del Traspaso
Antes de emitir comandos para el nodo Linux, el agente en macOS debe:
1. Ejecutar localmente `cargo build --release --workspace`.
2. Validar que `cargo test --workspace` pase al 100% (cero fallos, cero advertencias).
3. Confirmar que la suite C ABI compile sin inconsistencias de tipos.
4. Realizar `git push origin main` y verificar la sincronizacion del arbol remoto.
5. Proporcionar comandos formateados usando rutas relativas o variables de entorno (`$PWD`):
   - Detener procesos en segundo plano para evitar conexiones a instancias huerfanas (`killall -9 zen zen-bin firefox 2>/dev/null || true`).
   - Precargar el binario compilado mediante `LD_PRELOAD=$PWD/target/release/libhoney_shim.so`.
   - Comprobar la memoria del proceso con `pmap $(pgrep -f "zen" | head -n 1) | grep libhoney`.

### 4.2 Obligaciones del Agente Linux durante la Ejecucion
1. Sincronizar el repositorio con `git pull origin main` y verificar el hash del commit.
2. Compilar con `cargo build --release --workspace`.
3. Validar con `gcc -I/usr/include/freetype2 -Ltarget/release -lhoney_shim examples/test_comprehensive_abi.c -o /tmp/test_abi && LD_LIBRARY_PATH=target/release /tmp/test_abi`.
4. Ejecutar las pruebas de aplicaciones solicitadas.
5. Capturar la pantalla con los metodos de Wayland descritos en la seccion 5.
6. Subir las evidencias a `docs/issues/` mediante `git push origin main` para que el arquitecto en macOS las examine.

---

## 5. Estandar de Registro de Evidencias y Telemetria en Wayland

### 5.1 Politica Zero Bloat para Capturas de Pantalla
1. **Eliminacion de Evidencias Obsoletas:** Al validar una solucion, eliminar de inmediato las capturas viejas resueltas:
   ```bash
   rm -f docs/issues/*.png
   ```
2. **Nomenclatura Estandarizada:** Guardar unicamente capturas activas con nombres descriptivos en minúsculas:
   - `docs/issues/captura_escritorio_navegadores.png`
   - `docs/issues/captura_zen_whatsapp.png`
   - `docs/issues/captura_descendentes_linea_base.png`

### 5.2 Metodos de Captura en Wayland
Debido a que herramientas clasicas de X11 no capturan superficies Wayland, utilizar:
- **Metodo D-Bus GNOME Shell:**
  ```bash
  gdbus call --session \
    --dest org.gnome.Shell.Screenshot \
    --object-path /org/gnome/Shell/Screenshot \
    --method org.gnome.Shell.Screenshot.Screenshot true false \
    "$PWD/docs/issues/captura_escritorio_navegadores.png"
  ```
- **Metodo Portal XDG (Universal GNOME / KDE):**
  ```bash
  gdbus call --session \
    --dest org.freedesktop.portal.Desktop \
    --object-path /org/freedesktop/portal/desktop \
    --method org.freedesktop.portal.Screenshot.Screenshot "" "{'interactive': <false>}"
  ```
- **Metodos Nativos de CLI:**
  - En GNOME: `gnome-screenshot -f /ruta/al/archivo.png`
  - En wlroots (Sway/Hyprland): `grim /ruta/al/archivo.png`
  - En KDE Plasma: `spectacle -b -n -o /ruta/al/archivo.png`

---

## 6. Orden Secuencial de Documentacion y Memoria Tecnica

Cualquier modificacion o cierre de issue debe registrarse en el siguiente orden estricto:

```text
1. Modificacion de Codigo:
   Implementar y refactorizar en crates/honey-core o crates/honey-shim.

2. Verificacion Local:
   cargo test --workspace
   gcc ... examples/test_comprehensive_abi.c

3. Gestion de Evidencias:
   Registrar la nueva captura en docs/issues/ y eliminar las obsoletas.

4. Memoria Tecnica y Articulos:
   Actualizar la documentacion correspondiente en docs/articles/ y el roadmap en docs/plan/.

5. Sincronizacion de Commits:
   git add -A
   git commit --author="isgaar <may17jun2002@outlook.com>" -m "<tipo>(<alcance>): <descripcion concisa>"
   git push origin main
```

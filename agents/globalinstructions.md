# Global Instructions: Protocolo de Colaboracion y Gobernanza Multi-Agente

Este documento establece las directivas globales, la division de roles, las normas de codigo limpio y las politicas de documentacion que rigen el desarrollo del proyecto **Honey**.

---

## 1. Topologia de Agentes y Asignacion de Roles

El desarrollo de Honey se realiza mediante la coordinacion asincrona de dos agentes especializados. Cada agente tiene un ambito estricto de responsabilidad:

```
+-------------------------------------------------------------------------------+
|                       ARQUITECTURA DE TRABAJO MULTI-AGENTE                     |
+-------------------------------------------------------------------------------+
| AGENTE 1: MACOS (Arquitecto Lider y Guardian del Objetivo)                    |
| - Entorno: macOS Sequoia (x86_64, Darwin 24.6.0)                              |
| - Tarea Principal:                                                            |
|   1. Disenar la arquitectura matematica y en Rust puro (honey-core).          |
|   2. Definir la C ABI de FreeType 2.13.2 (honey-shim) con paridad Quartz.     |
|   3. Redactar los articulos cientificos, planes de diseno y conclusiones.     |
|   4. CONTROL Y MONITOREO CRITICO: Detectar y marcar inmediatamente cuando     |
|      el agente de Linux se desvia del objetivo o empieza a formar caminos     |
|      alternos producto de alucinaciones o parches provisionales. Reencauzar   |
|      siempre el desarrollo hacia la sustitucion nativa y limpia de FreeType.  |
+-------------------------------------------------------------------------------+
| AGENTE 2: LINUX (Ejecutor de Integracion y Banco de Pruebas Real)             |
| - Entorno: Fedora 40/41 (Wayland, SELinux Enforcing)                          |
| - Tarea Principal:                                                            |
|   1. Compilar y enlazar binarios en el hardware de produccion Linux.          |
|   2. Ejecutar la suite C ABI de integracion (test_comprehensive_abi.c).       |
|   3. Lanzar y probar aplicaciones reales (Zen Browser, Firefox, Nautilus).   |
|   4. Registrar capturas de pantalla reales en docs/issues/ y reportar salida. |
|   5. Ceñirse estrictamente a las directivas del Agente macOS sin improvisar   |
|      soluciones perifericas (no volver a parches de fontconfig, CSS o flags). |
+-------------------------------------------------------------------------------+
```

---

## 2. Reglas Inmutables de Proyecto (Zero Tolerance)

### 2.1 Prohibicion Absoluta de Emojis (Strict Zero Emojis)
Queda **estrictamente prohibido el uso de emojis, emoticonos o pictogramas Unicode** en:
- Todo el codigo fuente (Rust, C, Shaders, Makefiles, Scripts Bash).
- Mensajes de commit en Git.
- Documentacion tecnica (`.md`, `.txt`, `.spec`, `.json`).
- Respuestas directas al usuario en el chat.
*Motivo:* El proyecto mantiene un estricto rigor de ingenieria de sistemas de bajo nivel y publicacion cientifica.

### 2.2 Rutas Relativas Obligatorias en Documentacion
**Nunca utilizar rutas absolutas locales** (como rutas completas de disco o esquemas URI locales) dentro de ningun archivo Markdown del repositorio.
- Todo enlace entre documentos debe usar **rutas relativas** (por ejemplo: `[Font Parser](../crates/honey-core/src/font.rs)` o `[Plan](plan/PLAN_REEMPLAZO_NATIVO_FREETYPE.md)`).
- Las rutas absolutas se rompen al cargarse en GitHub, en servidores de integracion continua o en la maquina de otro desarrollador.

### 2.3 Identidad de Commits en Git
Todos los commits deben realizarse con la siguiente identidad explicitada mediante `--author`:
```bash
git commit --author="isgaar <may17jun2002@outlook.com>" -m "<tipo>(<alcance>): <descripcion concisa>"
```

### 2.4 Estandares de Codigo Limpio (Clean Code)
- **Rust Idiomatico:** Cero advertencias (`warnings`) del compilador.
- **Zero Allocations en Bucles Calientes:** El renderizado y la mezcla fotometrica no deben asignar memoria dinamica en el heap durante el rasterizado de glifos.
- **Manejo Seguro de Errores:** Evitar `unwrap()` y `panic!()` en `honey-shim`. Devolver siempre codigos de error de FreeType (`FT_Err_...` o 1) para garantizar que las aplicaciones host no sufran cierres inesperados.
- **Transparencia y Cero Hinchazon (*Zero Bloat*):** No acumular codigo muerto ni bibliotecas no esenciales. El binario debe mantenerse ligero y ultrarrapido.

---

## 3. Estado Real del Proyecto: Pre-Alpha Experimental

Es indispensable mantener una comunicacion honesta y precisa:
- **Honey no ha llegado a fase Beta.** Es un proyecto en fase **pre-alpha experimental**.
- El objetivo a largo plazo es ambicioso: sustituir completamente a FreeType (y ClearType) mediante paridad Quartz.
- Toda documentacion debe reflejar con claridad el estado actual: lo que ya funciona (rasterizador analitico, stem darkening, C ABI basica, soporte de streams web) y lo que aun esta en desarrollo activo (compositor lineal Wayland, integracion completa de perfiles GNOME/KDE).

---

## 4. Orden Secuencial de Documentacion y Guardado de Memoria

Cualquier actualizacion o resolucion de problemas debe seguir este orden estricto:

```text
1. Modificacion de Codigo:
   Implementar y probar en crates/honey-core o crates/honey-shim.

2. Verificacion de Paridad:
   cargo test --workspace && gcc examples/test_comprehensive_abi.c ...

3. Depuracion de Evidencias:
   Si se tomo una captura para resolver un issue visual, registrarla en docs/issues/
   y eliminar de inmediato las capturas viejas resueltas.

4. Registro de Memoria Tecnica:
   Actualizar los articulos correspondientes en docs/articles/ y el roadmap en docs/plan/.

5. Sincronizacion de Commits:
   git add -A
   git commit --author="isgaar <may17jun2002@outlook.com>" -m "..."
   git push origin main
```

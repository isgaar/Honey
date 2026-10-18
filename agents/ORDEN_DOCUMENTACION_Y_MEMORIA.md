# Orden de Documentacion, Convenciones de Git y Guardado de Memoria

Este protocolo establece las normas operativas obligatorias que todo agente (humano o sintetico) debe acatar al interactuar con el codigo, la documentacion, el historial de Git y la memoria tecnica del repositorio **Honey**.

---

## 1. Reglas Inmutables de Estilo y Autoria

### 1.1 Prohibicion Absoluta de Emojis (Strict Zero Emojis)
Queda **estrictamente prohibido el uso de emojis o pictogramas Unicode** en:
- Codigo fuente (Rust, C, Shaders, Scripts de Shell).
- Comentarios de codigo y docstrings.
- Archivos de documentacion (`.md`, `.txt`, `.spec`, `.json`).
- Mensajes de commit en Git.
- Respuestas directas al usuario en el chat.

*Justificacion:* El proyecto mantiene un estricto estandar de publicacion cientifica y de ingenieria de sistemas de nivel operativo.

### 1.2 Identidad de Commits de Git
Todo commit realizado en el repositorio debe atribuirse explicitamente al usuario configurado mediante la bandera `--author`:
```bash
git commit --author="isgaar <may17jun2002@outlook.com>" -m "<tipo>(<alcance>): <descripcion concisa>"
```
Queda prohibido realizar commits anonimos o con credenciales predeterminadas del entorno virtual.

### 1.3 Formato de Mensajes de Commit (Conventional Commits)
Los mensajes de commit deben seguir el estandar internacional:
- `feat(<componente>): ...` para nuevas funcionalidades.
- `fix(<componente>): ...` para correcciones de bugs o discrepancias de ABI.
- `docs(<componente>): ...` para documentacion tecnica o actualizacion de evidencias.
- `refactor(<componente>): ...` para reorganizacion sin cambio de funcionalidad.
- `test(<componente>): ...` para nuevas suites de verificacion o benchmarks.

---

## 2. Orden Secuencial de Documentacion

Cada vez que un agente implemente una mejora o resuelva un problema en el motor, debe actualizar los documentos siguiendo este orden jerarquico estricto:

```
Paso 1: Codigo y Pruebas
   ├── Implementar cambio en crates/honey-core o crates/honey-shim
   └── Validar suite local: cargo test --workspace && test_comprehensive_abi.c

Paso 2: Registro de Evidencias (docs/issues/)
   ├── Si se corrigio un bug visual, registrar la nueva captura de pantalla
   └── Eliminar las capturas antiguas que ya fueron resueltas

Paso 3: Actualizacion de Articulos y Perfiles (docs/articles/)
   └── Documentar la solucion matematica o arquitectonica en el articulo correspondiente

Paso 4: Actualizacion del Roadmap y Planes (docs/plan/)
   └── Actualizar el estado de los hitos en ROADMAP_HITOS_Y_FASES.md

Paso 5: Actualizacion del Historial Publico (CHANGELOG.md y README.md)
   └── Registrar la version y los puntos clave resueltos
```

---

## 3. Protocolo de Guardado de Memoria Tecnica

Cuando una sesion de trabajo avanza o se prepara para una compactacion de contexto, el agente en ejecucion debe:

1. **Sintetizar el estado en un registro de memoria tecnica**:
   - Resumen del problema resuelto y causa raiz diagnosticada.
   - Commit SHA resultante.
   - Archivos modificados y lineas clave intervenidas.
   - Comprobaciones matematicas y resultados de paridad de salida.
2. **Preservar la memoria en el repositorio**:
   La memoria tecnica no debe perderse en el historial volatil del chat. Debe quedar plasmada en los articulos de [docs/articles/](../docs/articles) y en los hitos de [docs/plan/](../docs/plan).
3. **Formato de Enlaces Clickables**:
   Toda referencia a archivos o simbolos debe formatearse con enlaces absolutos clickables mediante el esquema `file://`:
   `[nombre_archivo.ext](/ruta/absoluta/al/archivo)`

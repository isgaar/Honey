# Directorio de Gobernanza de Agentes y Memoria Tecnica de Honey

Este directorio define el protocolo formal de colaboracion, sincronizacion y gestion de memoria tecnica entre los diferentes agentes inteligentes que desarrollan y mantienen el proyecto **Honey**.

---

## 1. Topologia de Desarrollo Multi-Agente

El desarrollo de Honey se ejecuta mediante un modelo de dos nodos especializados conectados a traves del repositorio Git central en GitHub:

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

---

## 2. Indice de Protocolos para Agentes

1. **[ORDEN_DOCUMENTACION_Y_MEMORIA.md](file:///Users/ismael/Documents/Honey/docs/agents/ORDEN_DOCUMENTACION_Y_MEMORIA.md)**:
   Reglas inmutables para la redaccion de documentacion tecnica, convenciones obligatorias de commits, preservacion de contexto y actualizacion de memorias vivas.
2. **[PROTOCOLO_COORDINACION_AGENTES.md](file:///Users/ismael/Documents/Honey/docs/agents/PROTOCOLO_COORDINACION_AGENTES.md)**:
   Flujo de trabajo secuencial para el traspaso de instrucciones, validaciones previas y verificaciones posteriores entre el nodo macOS y el nodo Linux.
3. **[REGISTRO_EVIDENCIAS_Y_TELEMETRIA.md](file:///Users/ismael/Documents/Honey/docs/agents/REGISTRO_EVIDENCIAS_Y_TELEMETRIA.md)**:
   Estandar de captura de imagenes, telemetria de glifos, suites de paridad ABI C y limpieza de evidencias obsoletas.

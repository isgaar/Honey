# Protocolo de Coordinacion y Handshake entre Agentes (macOS <-> Linux)

Este documento especifica el protocolo de comunicacion asincrono, traspaso de instrucciones (*handshake*) y criterios de aceptacion entre el **Agente macOS** (desarrollo y analisis) y el **Agente Linux / Fedora** (pruebas en vivo e integracion).

---

## 1. El Ciclo de Relevos (Handshake Cycle)

```
[ Agente macOS ]                                          [ Agente Linux / Fedora ]
       |                                                             |
 1. Diagnostica y codifica en Rust                                   |
 2. Valida unit tests y C ABI local                                  |
 3. Commit y push a origin/main                                      |
 4. Genera bloque markdown de traspaso ------------> 5. Recibe bloque de instrucciones
                                                      6. git pull origin main
                                                      7. cargo build --release --workspace
                                                      8. Ejecuta test_comprehensive_abi.c
                                                      9. Lanza apps (Zen, Firefox, Nautilus)
                                                     10. Toma capturas en docs/issues/
                                                     11. Commit y push a origin/main
 13. Inspecciona captura y continua <---------------- 12. Emite reporte con salida real
```

---

## 2. Obligaciones Previas del Agente macOS antes del Traspaso

Antes de proporcionar un bloque de comandos para el nodo Linux, el agente en macOS debe:
1. Ejecutar localmente `cargo build --release --workspace`.
2. Validar que `cargo test --workspace` pase al 100% (cero fallos, cero warnings).
3. Compilar y ejecutar la suite C ABI contra las cabeceras disponibles en `/tmp` o locales.
4. Realizar `git push origin main` y verificar que el arbol remoto este limpio.
5. Formatear las instrucciones en un bloque unico de Markdown autoejecutable que:
   - Mate instancias previas antes de precargar el shim para evitar conexiones remotas al proceso viejo.
   - Use rutas absolutas correctas (`/home/ismael/Proyectos/Honey/...`).
   - Incluya comandos de verificacion de memoria (`pmap $(pgrep ...) | grep libhoney`).

---

## 3. Obligaciones del Agente Linux durante la Ejecucion

Al recibir las instrucciones, el agente en Linux debe:
1. Ejecutar `git pull origin main` y verificar el hash del commit descargado.
2. Compilar con `cargo build --release --workspace`.
3. Validar con `gcc ... test_comprehensive_abi.c` y confirmar que las 16 comprobaciones pasen.
4. Si se solicita captura de pantalla en Wayland:
   - Usar la interfaz nativa de D-Bus de GNOME Shell (`org.gnome.Shell.Screenshot`) o el portal de escritorio (`org.freedesktop.portal.Desktop`).
   - Comprobar que el archivo resultante no este corrupto y tenga tamano mayor a cero.
5. Hacer `git push` de las evidencias hacia `origin/main` para permitir al agente macOS inspeccionarlas visualmente.

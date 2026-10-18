# Diagnostico y Resolucion de Problemas (Troubleshooting)

Este documento reune las tecnicas de diagnostico, analisis de trazas y procedimientos de recuperacion ante cualquier anomalia durante el uso o instalacion de **Honey**.

---

## 1. Procedimiento de Recuperacion de Emergencia (Pantalla Negra tras `install-system`)

Si se ejecuto la instalacion de bajo nivel a nivel de sistema (`sudo honey install-system`) en una version anterior y el gestor de pantalla (GDM/SDDM) no inicia, quedando en pantalla negra con cursor:

### Causa comun en distribuciones con SELinux (Fedora / RHEL):
El archivo de biblioteca `/usr/local/lib64/libhoney_shim.so` no tenia la etiqueta de seguridad `lib_t`. El demonio de GDM (`xdm_t`) fue bloqueado por SELinux al intentar cargarlo.

### Pasos de recuperacion inmediata:
1. Acceder a una consola virtual TTY presionando `Ctrl + Alt + F3` (o `Ctrl + Alt + F4`).
2. Iniciar sesion con tu usuario y contrasena.
3. Ejecutar el comando de restauracion de Honey:
   ```bash
   sudo /home/ismael/Proyectos/Honey/bin/honey restore-system
   ```
4. Si el script no estuviera disponible, restaurar manualmente el cargador dinamico:
   ```bash
   sudo rm -f /etc/ld.so.conf.d/00-honey.conf
   sudo rm -f /usr/local/lib64/libhoney_shim.so /usr/local/lib64/libfreetype.so.6
   sudo ldconfig
   sudo systemctl restart gdm  # o sddm en KDE
   ```
5. Regresar al entorno grafico con `Ctrl + Alt + F1` o `Ctrl + Alt + F2`.

*Nota:* Las versiones modernas de `bin/honey` aplican automaticamente `restorecon -F` y `chcon -t lib_t`, ademas de realizar un smoke test en vivo con rollback automatico antes de salir, por lo que este fallo queda prevenido de raiz.

---

## 2. Diagnostico de Fuentes Web en Navegadores (Gecko / Chromium)

### Sintoma:
En WhatsApp Web, Slack o sitios con fuentes descargadas `@font-face`, los textos aparecen en blanco o con rectangulos vacios.

### Diagnostico:
Los navegadores basados en Gecko (Firefox y Zen Browser) empaquetan las fuentes web en memoria y llaman a `FT_Open_Face` con el flag `FT_OPEN_STREAM` (0x2).

### Verificacion:
1. Asegurate de tener una compilacion reciente de [crates/honey-shim](file:///Users/ismael/Documents/Honey/crates/honey-shim) posterior al commit `b277d1e`.
2. Verifica que el proceso del navegador tiene mapeada la biblioteca correcta:
   ```bash
   pmap $(pgrep -f "zen" | head -n 1) | grep libhoney
   ```
3. Si el navegador ya estaba abierto antes de lanzar el comando con `LD_PRELOAD`, este se conectara a la instancia en segundo plano sin precargar la biblioteca. Mata siempre los procesos huerfanos antes de probar:
   ```bash
   killall -9 zen zen-bin firefox 2>/dev/null || true
   LD_PRELOAD=/home/ismael/Proyectos/Honey/target/release/libhoney_shim.so zen-browser &
   ```

---

## 3. Diagnostico de Alineacion de Linea Base y Descendentes

### Sintoma:
Caracteres con astas descendentes (`'y'`, `'g'`, `'p'`, `'q'`) aparecen caidos o con huecos intercaracter en cadenas como *"502 Bad Gateway"*.

### Diagnostico:
El motor Skia calcula el rectangulo del glifo mediante `FT_Outline_Get_CBox` antes de llamar a `FT_Render_Glyph`. Si el rasterizador anade relleno artificial (`pad = 1`), Skia desalinea la celda del atlas.

### Verificacion:
Ejecuta la suite de paridad de caja delimitadora en [examples/test_comprehensive_abi.c](file:///Users/ismael/Documents/Honey/examples/test_comprehensive_abi.c):
```bash
./bin/honey test
```
Confirma que el paso 11 reporte exactamente el tamano sin padding (ej. `13x18 px` en Adwaita Sans 24px) y que todas las 16 comprobaciones emitan `[PASS]`.

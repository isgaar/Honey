# Estandar de Registro de Evidencias Visuales y Telemetria

Este documento estandariza como deben registrarse, nombrarse, auditarse y depurarse las evidencias graficas (capturas de pantalla) y telemetricas (matrices de glifos y benchmarks) en el repositorio.

---

## 1. Convencion para Evidencias Graficas (`docs/issues/`)

El directorio [docs/issues/](../docs/issues) almacena exclusivamente las capturas de pantalla activas que documentan un problema tipografico en curso o la validacion final de una correccion.

### 1.1 Reglas de Depuracion (Zero Bloat)
Para evitar que el repositorio de Git crezca innecesariamente con gigabytes de binarios PNG obsoletos:
1. **Eliminacion de capturas resueltas:**
   Al solucionar un problema de renderizado, las capturas de pantalla antiguas que documentaban la falla deben ser eliminadas del arbol de trabajo antes de subir las nuevas:
   ```bash
   rm -f docs/issues/*.png
   ```
2. **Nombres descriptivos y estandarizados:**
   Las capturas deben usar nombres descriptivos en minúsculas:
   - `captura_escritorio_navegadores.png`: Vista general del escritorio con navegadores y apps abiertas.
   - `captura_zen_whatsapp.png`: Detalle de WhatsApp Web o fuentes web.
   - `captura_descendentes_linea_base.png`: Detalle de la linea base y caracteres `'y'`, `'g'`, `'p'`, `'q'`.

---

## 2. Metodos Estandarizados de Captura en Linux / Wayland

En Fedora y distribuciones modernas con Wayland, las herramientas clasicas de X11 (`import`, `xwd`) fallan. Los agentes deben utilizar los siguientes metodos segun el entorno:

### 2.1 Metodo 1: D-Bus Directo de GNOME Shell (Recomendado en GNOME)
```bash
gdbus call --session \
  --dest org.gnome.Shell.Screenshot \
  --object-path /org/gnome/Shell/Screenshot \
  --method org.gnome.Shell.Screenshot.Screenshot true false \
  "$HOME/Proyectos/Honey/docs/issues/captura_escritorio_navegadores.png"
```

### 2.2 Metodo 2: Portal de Escritorio XDG (Agnostico para GNOME y KDE)
```bash
gdbus call --session \
  --dest org.freedesktop.portal.Desktop \
  --object-path /org/freedesktop/portal/desktop \
  --method org.freedesktop.portal.Screenshot.Screenshot "" "{'interactive': <false>}"
```

### 2.3 Metodo 3: Herramientas de Linea de Comandos Nativas
- En GNOME: `gnome-screenshot -f /ruta/al/archivo.png`
- En wlroots / Sway / Hyprland: `grim /ruta/al/archivo.png`
- En KDE Plasma: `spectacle -b -n -o /ruta/al/archivo.png`

---

## 3. Registro de Telemetria Tipografica

Para validar la paridad de glifos sin depender de inspeccion visual humana, los agentes deben recurrir a programas de validacion C contra la biblioteca compilada:
- Suite ABI global: [examples/test_comprehensive_abi.c](../examples/test_comprehensive_abi.c)
- Programa de prueba de glifo individual: extraccion de `bitmap_left`, `bitmap_top`, `width`, `rows`, `horiBearingY` y comparacion contra la salida de `libfreetype.so.6` oficial.

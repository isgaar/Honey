# Guia de Uso y Manual de Comandos de Honey

Este documento describe la interfaz de linea de comandos (CLI) de **Honey** (`bin/honey`), sus subcomandos, banderas y modos de operacion.

---

## 1. Referencia de Comandos del CLI (`bin/honey`)

```text
Uso: honey <subcomando> [opciones]

Comandos de Gestion del Motor Nativo:
  install-system       Instala Honey Rust Core como reemplazo nativo de libfreetype.so.6 (requiere sudo)
  status-system        Muestra el estado de enlace y resolucion dinamica de ejecutables del sistema
  restore-system       Restaura el sistema al motor FreeType original de la distribucion (requiere sudo)

Comandos de Perfil de Escritorio:
  profile set gnome    Aplica el perfil de optimizacion para GNOME (Mutter / GTK4 / GSK)
  profile set kde      Aplica el perfil de optimizacion para KDE Plasma (KWin / Qt6 / SDF)
  profile status       Muestra el perfil de escritorio actualmente activo

Comandos de Diagnostico y Aplicacion:
  status               Verifica el estado de las fuentes y enlaces del sistema
  apply                Aplica el perfil tipografico en espacio de usuario
  test                 Ejecuta la suite de pruebas de paridad C ABI (test_comprehensive_abi)
```

---

## 2. Ejemplos de Flujos de Trabajo Tipicos

### 2.1 Verificacion del Estado del Motor en el Sistema
Para comprobar si los ejecutables clave del escritorio (como `fc-match`, `alacritty`, `codium`, `gnome-shell`) estan enlazando activamente con Honey:
```bash
./bin/honey status-system
```

Salida esperada:
```text
Estado del reemplazo de sistema de Honey:
  ld.so configuracion: /etc/ld.so.conf.d/00-honey.conf -> /usr/local/lib64

Resolucion dinamica de ejecutables del sistema:
  fc-match            :   libfreetype.so.6 => /usr/local/lib64/libfreetype.so.6 (0x00007f4afff6e000)
OK Motor tipografico activo: Honey Rust Core (Paridad Quartz)
```

### 2.2 Seleccion y Activacion de Perfiles de Escritorio

Para estaciones de trabajo con **GNOME Desktop** (Fedora Workstation, Ubuntu, Arch GNOME):
```bash
./bin/honey profile set gnome
```
- Configura dilatacion compensada para el supersampling de Mutter.
- Asegura la alineacion de pitch divisible por 4 para Cairo y GSK.
- Optimiza para Adwaita Sans y Cantarell.

Para estaciones de trabajo con **KDE Plasma** (Kubuntu, Fedora KDE, Arch KDE):
```bash
./bin/honey profile set kde
```
- Emite contornos continuos optimizados para generacion de Signed Distance Fields en QtQuick.
- Desactiva el sobre-engrosamiento adaptandose al escalado geometrico nativo de KWin.
- Optimiza para Noto Sans y tipografias Breeze.

### 2.3 Ejecucion de Pruebas de Paridad C ABI
Para verificar que la biblioteca compartida mantiene compatibilidad con la especificacion FreeType 2:
```bash
./bin/honey test
```
Compilara y ejecutara [examples/test_comprehensive_abi.c](file:///Users/ismael/Documents/Honey/examples/test_comprehensive_abi.c), confirmando que las 16 comprobaciones de paridad de ABI (inicializacion, descomposicion de contornos, cajas delimitadoras CBox, glifos independientes y tablas SFNT) pasen al 100%.

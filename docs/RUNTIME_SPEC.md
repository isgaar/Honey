# Especificacion del runtime Honey

Esta especificacion describe el comportamiento actual. Es pequena a proposito:
sirve como base estable para extender el runtime.

## Objetivos

Honey busca ser:

- facil de instalar;
- facil de leer;
- facil de modificar;
- suficientemente pequeno para entenderlo completo;
- suficientemente ordenado para crecer sin volverse confuso.

## Modelo de ejecucion

Honey ejecuta un archivo `.hny` linea por linea.

El flujo es:

1. Leer una linea.
2. Quitar espacios al inicio y al final.
3. Ignorar lineas vacias.
4. Ignorar comentarios que empiezan con `#`.
5. Detectar el comando.
6. Ejecutar la accion.
7. Continuar con la siguiente linea.

## Sintaxis

La forma general es:

```text
comando argumentos
```

Ejemplos:

```honey
print Hola
env USER
sleep 1
exit 0
```

## Comentarios

Solo se reconocen comentarios de linea completa:

```honey
# Esto es un comentario
```

Por ahora no existen comentarios al final de una instruccion:

```honey
print Hola # esto se imprime como parte del texto
```

## Tipos de datos

Honey no tiene tipos formales todavia.

El runtime actual maneja:

- texto para `print`;
- nombres de variable para `env`;
- numeros positivos enteros o decimales para `sleep`;
- enteros positivos para `exit`.

## Errores

Honey falla rapido cuando encuentra algo que no entiende.

Ejemplo:

```honey
unknown value
```

Produce un error similar a:

```text
ERROR app.hny:1 comando desconocido: unknown
```

## Comandos reservados

Los comandos actuales son:

- `print`
- `env`
- `sleep`
- `exit`

Evita usar esos nombres para futuras construcciones incompatibles.

## Posibles extensiones futuras

Ideas naturales para siguientes versiones:

- variables propias del lenguaje;
- bloques;
- funciones;
- importacion de archivos;
- ejecucion de comandos externos con permisos claros;
- modo estricto;
- formato JSON para reportes;
- pruebas para programas `.hny`.

## Compatibilidad

Mientras Honey este en `0.x`, la sintaxis puede cambiar. Cuando llegue a `1.0`,
esta especificacion deberia endurecerse y separar claramente:

- comportamiento estable;
- comportamiento experimental;
- comportamiento obsoleto.


# Uso de Honey

Honey se usa con el comando `honey`.

## Ver ayuda

```bash
honey help
```

## Ver version

```bash
honey version
```

## Crear un programa

```bash
honey new app.hny
```

Eso crea:

```honey
# Programa Honey
print Hola desde Honey
env USER
```

## Ejecutar un programa

```bash
honey run app.hny
```

Tambien puedes ejecutar el ejemplo incluido:

```bash
honey run examples/hello.hny
```

## Revisar entorno

```bash
honey doctor
```

Este comando imprime informacion util para depurar rutas y version de Bash.

## Formato `.hny`

Un archivo Honey es texto plano. Cada linea contiene una instruccion.

Ejemplo:

```honey
# Las lineas que empiezan con # son comentarios
print Iniciando
env HOME
sleep 1
print Listo
```

Reglas actuales:

- Las lineas vacias se ignoran.
- Las lineas con `#` al inicio se ignoran.
- Los comandos se evaluan de arriba hacia abajo.
- Si aparece un comando desconocido, Honey termina con error.

## Comandos del lenguaje

### print

Imprime texto.

```honey
print Hola mundo
```

Salida:

```text
Hola mundo
```

### env

Imprime el valor de una variable de entorno.

```honey
env HOME
```

Si la variable no existe, imprime una linea vacia.

### sleep

Pausa la ejecucion durante un numero de segundos.

```honey
sleep 1
```

Tambien acepta decimales:

```honey
sleep 0.5
```

### exit

Termina la ejecucion con un codigo numerico.

```honey
exit 0
```

## Codigos de salida

- `0`: ejecucion correcta.
- `1`: error general.
- Otro codigo: puede venir de `exit CODIGO` dentro del programa.


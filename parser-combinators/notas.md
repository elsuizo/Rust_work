# Vamos a aprender parser combinators con Rust

Vamos a practicar con una version simplificada de XML, que se ve mas o menos como
esto:

```xml
<parent-element>
   <single-element attribute="value" />
</parent-element>
```

Como vemos los elementos de XML comienzan con el simbolo `<` y un identificador
consite en una letra seguido de cualquier numero o letra, numero y `-`. Esto es
seguido por un espacio en blanco y una lista opcional de pares que son los atributos
otro atributo definido como el que definimos antes, seguido de un `=` y un `""`.
Finalmente existe un caracter que indica que ha finalizado `/>` y que el elemento
no tiene mas hijos, o si hubiera aparecido un `>` solo significaria que tenemos
una secuencia de elementos hijos a continuacion. Esto es solo lo que vamos a
soportar para esta version simple, ya que la verdadera tiene un monton de cosas
mas. Vamos a parsear los elementos dentro de una struct que sera algo asi:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
struct Element {
   name: String,
   attributes: Vec<(String, String)>,
   children: Vec<Element>,
}
```

## Definiendo el parser

Parsear es un proceso donde derivamos una structura desde una corriente de datos
Podemos pensar como una simplificacion a una funcion que toma una entrada y retorna
como salida parseada con lo que resta de la entrada para ser parseada o un error
como "no puedo parsear esto!!!", osea algo que consume una entrada y produce o un error
o una salida parseada con lo que resta para parsear. Podemos escribir un prototipo de
funcion en el lenguaje de Rust:

```rust
Fn(Input) -> Result<(Input, Output), Error>
```

Osea que podemos escribir en funcion de types de Rust conocidos como:

```rust
Fn(&str) -> Result<(&str, Element), &str>
```

Aca la salida que queremos es la estructura que pusimos anteriormente, se elige
el type `&str` en lugar de `&[u8]` porque es mejor cuando se trabaja con UTF-8
y tiene mas metodos que hacen mas facil parsear

## Nuestro primer parser

Vamos a escribir nuestro primer parser el cual solo mira en el primer caracter
en el string y decide cuando o no es una letra `a`

```rust
fn the_letter_a(input: &str) -> Result<(&str, ()), &str> {
   match input.chars().next() {
      Some('a') => Ok((&input['a'.len_utf8()..], ())),
      _         => Err(input)
   }
}
```

Como dijimos tenemos como entrada un slice de string y como salida tenemos un
`Result` de o un `(&str, ())` o un type de error que en este caso es solo un
slice de string `&str`. Lo que retornamos es la parte que le sigue a lo que hemos
parseado y `()` que es el type unitario ya que como hemos encontrado la letra `a`
no necesitamos devolverla en este caso. Como vemos en el codigo se tiene en cuenta
que el ascii proviene de una secuencia de bytes UTF-8 validos y por eso usa
la funcion para obtener cuanto tiene que tomar del slice `len_utf8()`. No necesitamos
esto para nuestro parser de XML, pero la primera cosa que tenemos que ver es como
vemos en el ejemplo simple de XML es el `char` `<` `/` y `=`, por ello podriamos
hacer que la funcion tome como parametro el `char` que necesitamos

## Un constructor de parsers

Podemos ir un paso mas y construir una funcion que produzca un parser para un `string`
statico de cualquier largo, no solo un solo caracter ya que un slice de string es
un UTF-8 valido y asi no tenemos que pensar en los problemas de unicode

```rust
fn match_literal(expected: &'static str) -> impl Fn(&str) -> Result<(&str, ()), &str> {
   move |imput| match input.get(0..expected.len()) {
      Some(next) if next == expected => {
         Ok((&input[expected.len()..], ()))
      },
      _ => Err(input),
   }
}
```

Entonces si ahora miramos los types, tenemos como entrada a una slice string
`expected` y como salida tenemos una familia de funciones que cumplan con el
criterio que que la entrada sea un `&str` y la salida sea el mismo `Result` que
en el caso anterior, no podemos retornar el slice de string directamente porque
no sabemos cual es basicamente(y eso es lo poderoso) pero lo que hacemos es que
expresar en la condicion del `match` `if next == expected`

Osea lo que hace la funcion que creemos es consumir el slice string y devolver
lo que nos resta del string que le pasemos

 - Ejercicio:
Can you find a method on the str type in the standard library that would let
you write `match_literal()` without having to do the somewhat cumbersome get
indexing?

## Un parser para algo un poco menos especifico

Entonces si queremos parsear '<', '>', '='. Estamos casi, lo que nos falta es
ahora reconocer despues de el `char` que abre '<' es el nombre. No podemos hacer
eso con una simple comparacion de strings, pero podriamos hacerlo con expresiones
regulares, pero es mucho para este simple caso.
Recordemos que la regla para el identificador del nombre es: una letra alfabetica
seguida por zero o mas caracters alfabeticos o un numero seguido de un '-'

```rust
fn indentifier(input: &str) -> Result<(&str, String), &str> {
   let mut matched = String::new();
   let mut chars = input.chars();

   match chars.next() {
      Some(next) if next.is_alfabetic() => matched.push_str(next),
      _                                 => return Err(input)
   }

   while let Some(next) = chars.next() {
      if next.is_alphanumeric() || next == '-' {
         matched.push(next);
      } else {
         break;
      }
   }

   let next_index = matched.len();
   Ok((&input[next_index..], matched))
}
```

Con esto tenemos el parser para la primer campo de la estructura `Element` que
habiamos declarado, tambien nos sirve para parsear la primer parte de cualquier
`attributes`, lo vemos en los tests

## Combinadores

Ahora que podemos parsear el simbolo que abre una secuencia de XML '<' y que podemos
parsear los indentificadores que le siguen. Lo que podemos hacer es otro constructor
de funciones, pero uno que tome dos "parsers" como entrada y que retorne un nuevo
"parser" el cual parsea a ambos en orden. En otras palabras un combinador de parsers
porque este combina dos parsers en uno nuevo.

```rust
fn pair
```

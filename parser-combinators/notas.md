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
fn pair<P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Fn(&str) -> Result<(&str, (R1, R2)), &str>
where
    P1: Fn(&str) -> Result<(&str, R1), &str>,
    P2: Fn(&str) -> Result<(&str, R2), &str>,
{
    move |input| match parser1(input) {
        Ok((next_input, result1)) => match parser2(next_input) {
            Ok((final_input, result2)) => Ok((final_input, (result1, result2))),
            Err(err) => Err(err),
        },
        Err(err) => Err(err)
    }
}
```

Primero miramos los types y vemos que la funcion es generica sobre cuatro types
los types de los parsers y los types de retorno de cada uno de ellos, como
vemos tambien el type de retorno es una tupla con los types de retorno de cada
uno de los parsers. Mirando el codigo de la funcion vemos que lo que hace es
exactamente eso comienza corriendo el primer parser en la `input` luego el
segundo parser y luego combinamos los dos resultados en una tupla y si alguno
de los dos falla en el camino retornamos inmediatamente con el error que ha
sucedido.  De esta manera podemos combinar nuestros dos parsers anteriores,
`match_literal` y `identifier` y asi podemos parsear la primera parte de el
XML. Veamos con los test como seria

Parece que anda!!! Pero mirando el type de retorno `((), String)` es obvio que
lo que nos interesa es el lado derecho de el. Este es casi siempre el caso algunos
de nuestros parsers solo matchean patrones en la entrada sin producir valores
y las salidas pueden ser ignoradas. Para acomodar esta situacion podemos usar
el `pair` combinator para escribir otros dos combinators: `left` que va a descartar
los resultados de el primer parser y solo retornara los resultados del segundo y
su opuesto `right` que es el que podriamos haber usado en el test anterior en
lugar de `pair`

## Entra en juego el functor

Pero antes de que continuemos, vamos a introducir otro combinator que nos va a
hacer escribir a estos dos de manera mas simple: `map`

Este combinator tiene un solo proposito: cambiar el type de el resultado. Por
ejemplo digamos que tenemos un parser que retorna `((), String)` y lo que queremos
es que retorne sea `String`. Para hacer esto, le pasamos una funcion que sabe como
comvertir desde el type original a el nuevo en nuestro ejemplo:
`|(_left, right)| right`. Generalizando aun mas, si quisieramos
`Fn(A) -> B` donde A es el type del resultado original del parser y B es el nuevo

```rust
fn map<P, F, A, B>(parser: P, map_fn: F) -> impl Fn(&str) -> Result<(&str, B), &str>
where
    P: Fn(&str) -> Result<(&str, A), &str>,
    F: Fn(A) -> B,
{
    move |input| match parser(input) {
        Ok((next_input, result)) => Ok((next_input, map_fn(result))),
        Err(err)                 => Err(err),
    }
}
```
Y ahora que dicen los types??? `P` is nuestro parser. El retorna `A` y nuestra
fn de mapeo transforma `A` en `B`

Podemos acortar un poco la funcion ya que esto que llamamos `map` resulta ser algo
comun con `Result`s que lo implementa:

```rust
fn map<P, F, A, B>(parser: P, map_fn: F) -> impl Fn(&str) -> Result<(&str, B), &str>
where
    P: Fn(&str) -> Result<(&str, A), &str>,
    F: Fn(A) -> B,
{
    move |input|
        parser(input).map(|(next_input, result)| (next_input, map_fn(result)))
}
```

A este patron se lo llama *functor* en los ambientes de Haskell y las matematicas
que lo soportan que son teoria de las categorias. Si tenemos algo de type `A` y
tenemos una funcion disponible que podemos pasar una funcion que va desde `A` ---> `B`


## Tiempo para representar las cosas en un trait

Como vimos estamos repitiendo la estructura de los types de nuestro parser:
`Fn(&str) -> Result<(&str, Output), &str>`. Podemos introducir un trait para que
las cosas se vean mejor. Pero primero podemos hacer typealias para que tengamos
que escribir menos:

`type ParseResult<'a, Output> = Result<(&'a str, Output), &'a str>;`

Y ahora para el trait, lo que necesitamos es poner el lifetime que pusimos en el
alias, por ejemplo:

```rust
trait Parser<'a, Output> {
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output>;
}
```

Como vemos solo tiene un metodo por ahora: `parse()` el cual nos debe parecer
familiar ya que es el mismo como el que escribimos como funcion de parser.
Para hacer esto mas facil podemos implementar este trait para cualquier funcion
que matchee los types de parser:

```rust
impl<'a, F, Output> Parser<'a, Output> for F
where
    F: Fn(&'a str) -> ParseResult<Output>,
{
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self(input)
    }
}
```
De esta manera no solo podemos pasarle la misma funcion que le estuvimos pasando
sino que ademas dejamos abierta la posibilidad de utilizar otros parsers, pero
capaz lo mas importante es que nos permite ahorrarnos escribir esas signaturas
cada vez
Ahora podemos reescribir la funcion map de nuevo:

```rust
fn map<'a, P, F, A, B>(parser: P, map_fn: F) -> impl Parser<'a, B>
where
    P: Parser<'a, A>,
    F: Fn(A) -> B,
{
    move |input|
        parser.parse(input).map(|(next_input, result)| (next_input, map_fn(result)))
}
```

Y podemos reescribir la funcion `pair` de la misma manera compacta:

```rust
fn pair<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, (R1, R2)>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    move |input| {
        parser1.parse(input).and_then(|(next_input, result1)| {
            parser2.parse(next_input).map(|(last_input, result2)| (last_input, (result1, result2)))
        })
    }
}
```

El metodo `and_then` es similar a `map` sino tendriamos que hacer dos match que lo
unico que hacen es mappear `None` con `None` y `Ok` con `Ok`. Luego lo que vamos
a hacer es los combinators `left` y `right`

## `left` y `right`

Ahora que ya tenemos a `map` y `pair`, podemos escribir a `right` y `left` de manera
bien compacta:

```rust
fn pair<'a, P1, P2, R1, R2>(parser1: P1, parser2: P2) -> impl Parser<'a, (R1, R2)>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    move |input| {
        parser1.parse(input).and_then(|(next_input, result1)| {
            parser2.parse(next_input).map(|(last_input, result2)| (last_input, (result1, result2)))
        })
    }
}
```
Ahora tenemos que actualizar los dos parser para que usen `Parser` y `ParserResult`

```rust
fn match_literal<'a>(expected: &'static str) -> impl Parser<'a, ()> {
    move |input: &'a str| match input.get(0..expected.len()) {
        Some(next) if next == expected => Ok((&input[expected.len()..], ())),
        _                              => Err()
    }
}
```
Para `identifier` solo tenenmos que cambiar el type de retorno, los lifetimes
no hace falta que lo pongamos

## Uno o mas

continuemos parseando los elementos del tag, ya tenemos el simbolo que abre '<'
y el indentificador, que sigue?

Necesitamos poder parsear `<element      attribute="value"/>` que es una sintaxis
valida, osea que tenemos que aceptar cualquier cantidad de espacios en blanco
entre el elemento y el identificador. Esta es una buena oportunidad para pensar
sobre cuando podemos escribir un combinator que expresa esa idea de uno o mas parsers
Hemos tratado con esto en el parser `identifier`, pero todo fue manual, sorprendentemente
el codigo que es mas general no es muy diferente del anterior

```rust
fn one_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
where
    P: Parser<'a, A>,
{
    move |mut input| {
        let mut result = Vec::new();
        if let Ok((next_input, first_item)) = parser.parse(input) {
            input = next_input;
            result.push(first_item);
        } else {
            return Err(input);
        }

        while let Ok((next_input, next_item)) = parser.parse(input) {
            input = next_input;
            result.push(next_item);
        }
        Ok((input, result))
    }
}
```

Primero que nada vemos que el type de retorno del parser que estamos construyendo
desde `A` y el type de retorno del parser combinado es `Vec<A>` cualquier numero
de `A`s. Como vemos el codigo se ve muy similar al que hicimos para `identifier`
Primero parseamos el primer elemento y si no hay nada alli retornamos un error
Luego parseamos la mayor cantidad de elementos que podamos hasta que el parser falla
y ahi retornamos el vector con los elementos collectados. Mirando ese codigo, como
seria adaptarlo para la idea de cero o mas?. Lo unico que necesitamos es remover
la primer llamada del parser

```rust
fn zero_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
where
    P: Parser<'a, A>
{
    move |mut input| {
        let mut result = Vec::new();

        while let Ok((next_input, next_item)) = parser.parse(input) {
            input = next_input;
            result.push(next_item);
        }
        Ok((input, result))
    }
}
```
Como vemos las diferencias entre los dos parsers es que `one_or_more` ve a un string
vacio como un `Err` porque necesita ver al menos un patron en el, pero para `zero_or_more`
un string vacio solo significa el caso zero que no es un error

A este punto parece razonable que podamos generalizar a estos dos parsers, porque uno es
una copia exacta del otro con un poco menos de codigo. Es tentador pensar que podemos expresar
a `one_or_more` en terminos de `zero_or_more` con algo como esto:

```rust
fn one_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
where
    P: Parser<'a, A>
{
    map(pair(parser, zero_or_more(parser)), |(head, mut tail)| {
        tail.insert(0, head);
        tail
    })
}
```

## Un combinator de predicados

Ahora que podemos parsear los espacios en blanco con `one_or_more` y parsear los
atributos con `zero_or_more`. Lo que estamos buscando es una secuencia de cero o mas
ocurrencias de uno o mas espacios en blanco seguidos por un atributo. Por ello necesitamos
un parser para un espacio en blanco simple primero. Podemos hacerlo de tres maneras:
 - Una es con el parser `match_literal(" ")` pero lo malo es que un cambio de linea
   tambien es un espacion en blanco, tab tambien es un espacio en blanco y muchos
   otros unicodes raros tambien se toman como espacios en blanco
 - Podemos escribir un parser que consuma cualquier tipo de espacios en blanco, usando la
   funcion de la libreria estandar `is_whitespace()` como lo usamos cuando escribimos
   el `identifier` antes
 - Podemos ser mas claros, podemos escribir un parser para cualquier `char` `any_char`
   el cual retorna un simple `char` siempre que haya uno a la izquierda del input y un
   combinator `pred` el cual toma un parser y una de estas funciones de predicado
   y combina las dos como: `pred(any_char, |c| c.is_whitespace())`. Esto tiene el bonus
   de que es muy simple para escribir el parser final.
   El parser `any_char` es simple como un parser, pero tenemos que tener en cuenta
   las complicaciones que traen los utf8

```rust
fn any_char(input: &str) -> ParseResult<char> {
    match input.chars().next() {
        Some(next) => Ok((&input[next.len_utf8()..], next)),
        _          => Err(input)
    }
}
```

y el combinator `pred` llamamos al parser, entonces llamamos a la funcion de predicado

```rust
fn pred<'a, P, A, F>(parser: P, predicate: F) -> impl Parser<'a, A> where
    P: Parser<'a, A>,
    F: Fn(&A) -> bool,
{
    move |input| {
        if let Ok((next_input, value)) = parser.parse(input) {
            if predicate(&value) {
                return Ok((next_input, value));
            }
        }

        Err(input)
    }
}
```

Con estos podemos entonces escribir el parser `whitespace_char`:

```rust
fn whitespace_char<'a>() -> impl Parser<'a, char> {
    pred(any_char, |c| c.is_whitespace())
}
```

Y ahora que tenemos esta funcion a disposicion podemos expresar la idea que estabamos buscando
"uno o mas espacios en blanco" y su idea hija que era: "cero o mas espacios en blanco"

```rust
fn space1<'a>() -> impl Parser<'a, Vec<char>> {
    one_or_more(whitespace_char())
}

fn space0<'a>() -> impl Parser<'a, Vec<char>> {
    zero_or_more(whitespace_char())
}
```

## String entre ""

con todo esto podemos al menos parsear esos atributos ???. Si lo unico que nos falta
es asegurarnos de tener todos los parsers individuales para los componentes de los
atributos, ya tenemos `identifier` para el nombre del atributo

```rust
fn quoted_string<'a>() -> impl Parser<'a, String> {
    map(
        right(
            match_literal("\""),
            left(
                zero_or_more(pred(any_char, |c| *c != '"')),
                match_literal("\""),
            )
        ),
        |chars| chars.into_iter().collect()
    )
}
```

## Como ultimo, parseando atributos

Como ya podemos parsear espacios en blanco, identificadores, signos como `=` y
string entre `""` entonces finalmente es todo lo que necesitamos para parsear
los atributos.
Primero necesitamos escribir un parser para un par de atributos, los vamos a guardar
como un `Vec<(String, String)>`

```rust
fn attribute_pair<'a>() -> impl Parser<'a, (String, String)> {
    pair(identifier, right(match_literal("="), quoted_string()))
}
```

## Ya estamos cada vez mas cerca...

A este punto las cosas parecen encaminarse a lo que queriamos lograr que es parsear
el XML. Tenenemos dos tipos de elementos con los cuales tenemos que lidiar: el elemento
simple y el elemento compuesto con hijos.
Hagamos el elemento simple primero:
Podemos escribir un parser que tenga en cuenta lo que tienen los dos en comun:
el simbolo de apertura '<' el elemento de nombre y los atributos. Veamos si podemos
obtener el type de `Result` como `(String, Vec<(String, String)>)` como resultado
de dos simples combinators

```rust
fn element_start<'a>() -> impl Parser<'a, (String, Vec<(String, String)>)> {
    right(match_literal("<"), pair(identifier, attributes()))
}
```

Con esto podemos hacer rapidamente una funcion que

```rust
fn single_element<'a>() -> impl Parser<'a, Element> {
    map(
        left(element_start(), match_literal("/>")),
        |(name, attributes)| Element {
            name,
            attributes,
            children: vec![]
        },
    )
}
```

El type de retorno de `single_element` es tan complicado que el compilador nos avisa
de que su size es muy grande(ojo que ami no me paso es lo que dice el que escribio
el blog), no podemos ignorar este problema mas si queremos hacer una libreria
seria(dice que mejor que antes que continuemos que comentemos las dos ultimas
funciones hasta que se arregle el problema)

## Hasta el infinito y mas alla

Si alguna vez intentaste escribir un type que sea recursivo en Rust ya sabras cual
es la solucion a nuestro problema. Un ejemplo sencillo de un type que es recursivo
es el siguiente, una lista enlazada que la podemos expresar de la siguiente manera


```rust
enum List<A> {
   Cons(A, List<A>),
   Nil
}
```
Este codigo no compila porque tiene un size que es infinito para el compilador

La solucion es emplear un poco de indireccion. En lugar de nuestra lista sea un
`List::Cons` necesitamos que sea un elemento del type A con un puntero a una lista
del type A, osea:

```rust
enum List<A> {
   Cons(A, Box<List<A>>),
   Nil,
}
```
Otra cosa interesante de `Box` es que el type que ponemos dentro de el puede ser
abstracto o sea que en lugar de tener que lidiar con nuestros super complicados
types de las funciones de parseo podemos dejar que el chequeo de types sea simplemente
con un `Box<dyn Parser<'a, A>>`. Esto es genial pero cual es el lado malo de esto???
Bueno al agregar un nivel de indireccion hacemos que el sistema sea mas lento
y ademas el compilador puede perder alguna oportunidad de hacer alguna optimizacion
agresiva(como lo puede hacer cuando tiene al type en si delante de el y no uno abstracto)

Entonces hagamos esos cambios para que nuestro parser sea un `BoxedParser`

```rust

```


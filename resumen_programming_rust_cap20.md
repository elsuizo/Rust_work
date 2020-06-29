# Cap: 20 Macros

Como sabemos Rust soporta macros, que es una manera de extender el lenguaje
en una manera que nos permite hacer cosas que no podriamos con funciones o
metodos comunes. Por ejemplo la macro `assert_eq!` que es util cuando hacemos
algun test. Esto se podria haber hecho con una funcion generica, pero
`assert_eq!` hace mas cosas bajo del capot, como por ejemplo cuando el test
falla tira un error con el nombre del archivo en donde fallo y el numero de
linea de ese archivo, las funciones comunes no pueden tener acceso a esa
informacion porque la manera que trabajan es totalmente diferente. Los macros
son como una especie de escritura manual. Durante la compilacion, antes de
que los types sean chequeados y muuucho antes de que el codigo maquina sea
generado, cada macro llama a su version expandida, esto es reemplaza los
lugares que le hemos dicho antes con codigo de Rust. Por ejemplo si tenemos
`assert_eq!(gcd(6,10), 2);` se expandira en lo siguiente:

```rust
match (&gcd(6, 10), &2) {
   (left_val, right_val) => {
      if!(*left_val == *right_val) {
         panic!("assertion failed: `(lef == val)`,\
               (left: `{:?}`, right: `{:?}`)", left_val, right_val);
      }
   }
}
```

## Basicos de los macros

Con `macro_rules!()` es la principal manera de definir macros en Rust,
notemos que no hay un `!` cuando estamos definiendo la macro, solo lo usamos
cuando estamos llamando a una macro. No todas las macros se definen asi, solo
unas pocas, como `file!`, `line!` y `macro_rules!` que son construidas dentro
del compilador y toman otro metodo llamado "procedural macros".

Una macro definida con `macro_rules!` trabaja enteramente con "pattern
matching"(estas palabras es mejor no traducirlas porque son el core del
lenguaje) el cuerpo de una macro es solo una serie de reglas:

```rust
(pattern1) => (template1);
(pattern2) => (template2);
...
```

La version que vimos antes de `assert_eq!()` tiene un solo ~pattern~ y por
ello un solo `template`. Podemos usar cualquier tipo de brackets conocidos
como `{}`, `[]` o `()`. Pero por convencion usamos parentesis cuando llamamos
a `assert_eq!()` un brackets cuadrado cuando creamos un vector con `vec![]` y
un brackets de cierre(no se como mierda traducir "curly") para cuando
llamamos `macro_rules!{}`

## Basicos de las expansiones en las macros

Rust expande los macros muy temprano durante la compilacion. El compilador
lee nuestro codigo desde el principio hasta el final, definiendo y
expandiendo macros mientras lo va recorriendo. Por ello no podemos llamar a
un macro antes de que sea definida(en lugar de las funciones que si podemos
ponerlas en donde queramos). Cuando Rust expande la macro `assert_eq!()` lo
que pasa internamente es muy parecido a lo que pasa cuando evalua una
expresion de un `match`. Rust primero matchea los argumentos en contra de el
patron que esta en la macro
Los patrones de macros son como un mini lenguaje dentro de Rust. Son
esencialmente expresiones regulares para matchear codigo. Pero donde las
expresiones regulares operan sobre caracteres, los patrones operan sobre
"tokens" (numeros, nombres, marcas de puntuacion, etc...) que son los bloques
de construccion de los programas de Rust. Esto quiere decir que podemos dejar
comentarios dentro de los macros para que sean mas entendibles y no seran
tomados como parte del patron. Otra diferencia importante entre las
expresiones regulares y los patrones de macros es que los parentesis,
brackets, y braces siempre ocurren en pares en Rust. Esto es chequado antes
de que el macro sea expandido.

En este ejemplo, nuestro patron contiene `left:$expr` que le dice a Rust que
matchee una expresion (en este caso, `gcd(6, 10)`) y que la asigne el nombre
`left` de la misma manera asigna a toda la expresion que matchea con
"expression2" y le asigna el nombre `right`, los dos fragmentos de codigo en
este patron son del type `expr:` por ello esperan expresiones. Dado que este
patron matchea todos los argumentos, Rust expande el template
correspondientes
Los macros de Rust son muy parecidos a todos los frameworks que existen para
rellenar templates en el mundo de la programacion Web, la diferencian
sustancial es que la salida es codigo de Rust.

## Consecuencias no deseadas

Poner fragmentos de codigo dentro de templates es sutilmente diferente de
hacerlo con codigo comun y corriente que funciona con valores. Estas
diferencias no son muy obvias al principio. Mirando mejor la macro que vimos
anteriormente, veamos algunas de estas diferencias:
Primero, porque esta macro cra dos variables que llama `left_val` y
`right_val`, existe alguna razon por la cual no podamos escribir:


```rust
if !($left == $right) {
   panic!("assertion failed: `(left == right)`\
         (lef: `{:?}`, right: `{:?}`)", $left, $right)
}
```

Para contestar esta pregunta hagamos un juego mental de expandir la llamada
a la macro siguiente: `assert_eq!(letters.pop(), Some('z'))`, lo que quiere
evitar el que escribio esa macro es tener que evaluar la expresion tantas
veces como sea que se las llame(en este caso dos) por eso crea las variables
para guardar el estado de las mismas y no tener que repetir las evaluaciones,
y lo que es mas importante cuando evaluemos nuevamente a `letters.pop()` no
va a repetir el mismo dato(si puede ser el mismo resultado)!!!.

Segundo, Porque esta macro presta referencias a los valores de `left` y
`right`, porque no los guardar simplemente como valores asi:


```rust
macro_rules! bad_assert_eq {
   ($lef:expr, $right:expr) => ({
         match ($lef, $right) {
         (left_val, right_val) => {
         if !(left_val == right_val) {
         panic!("assertion failed ...")
         }
         }
         }
         })
}
```

Para el caso particular que estamos viendo funcionaria sin problemas pero que
pasa si le pasamos un `String` por ejemplo, esto haria que las variables
muevan los valores fuera de las variables que le pasamos!!!. Por eso para ser
los mas genericos posibles el que escribio la macro hizo que sean
referencias. A diferencia de lenguajes como C o C++ que los macros no gozan
de ningun chequeo de nada, el ejemplo tipico es:

`#define ADD_ONE(n) n + 1`

Esa macro produce resultados muy locos en llamadas como `ADD_ONE(1) * 10` o
`ADD_ONE(1<<5)`, que para "solucionar" debemos poner mas parentesis en
nuestra macro, pero lo peor es que el codigo compila y genera valores pero
que no son los que esperamos. En Rust en cambio no es necesario ya que las
macros estan mejor integradas en el lenguaje. Rust sabe cuando esta cargando
con una expresion y por ello efectivamente agrega los parentesis cuando esta
copiando una expresion en otra

## Repeticion

El macro de la libreria estandar `vec!` viene en dos formas:

```rust
// repetir un valor N veces
let buffer = vec![0_u8; 1000];

// una lista de valores separados por comas
let numbers = vec!["udon", "ramen", "soba"];

Puede implementarse asi:
```

```rust
macro_rules! vec {
   ($element:expr; $n:expr) => {
      ::std::vec::from_elemen($element, $n)
   };
   // este patron es una lista de items
   ($($x:expr), *) => {
      <[_]>::into_vec(Box::new([$($x), *]))
   };
   ($($x:expr),+,) => {
      vec![$($x), *]
   };
}
```

Como vemos en esta macro hay tres reglas. Cuando Rust expande una llamada a
un macro como `vec![1, 2, 3]` comienza tratando de matchear el argumento
`1, 2, 3` con el patron de la primera regla que como no cumple el patron
falla ya que el paton requiere que el separador sea un `;`, entonces se va a
la segunda regla y asi sucesivamente hasta que si no encuentra el patron
correcto emite un error como debe ser...
La segunda regla que tiene un patron que no vimos hasta ahora:
`$($x:expr),*,` usa la caracteristica de repeticion, este matchea 0 o mas
expresiones, separadas por comas. Mas generalmente la sintaxis es:

```rust
$(PATTERN),*
```
Es usada para matchear cualquier lista de argumentos separados por comas,
donde cada elemento en la lista matchea `PATTERN`. El `*` tiene el mismo
significado que con las expresiones regulares ("0 o mas"), pero aunque es
cierto que las expresiones regulares no tienen un operador especial `,*`
repetidor. Podemos usar `+` para requerir al menos un match. La siguiente
tabla nos da todas las posibilidades de patrones de repeticion


 | Patron           | Significado                                |
 | ---              | ---                                        |
 | `$(...)*`        | Matchea 0 o mas veces sin separador        |
 | `$(...),*`       | Matchea 0 o mas veces, separados por ,     |
 | `$(...);*`       | Matchea 0 o mas veces, separados por ;     |
 | `$(...)+`        | Matchea 1 o mas veces sin separador        |
 | `$(...),+`       | Matchea 1 o mas veces separados por ,      |
 | `$(...);+`       | Matchea 1 o mas veces separados por ;      |

El fragmento de codigo `$x` no es solo una expresion. El template para esta
regla usa la sintaxis de repeticion tambien:

```rust
<[_]>::into_vec(Box::new([$($x),*]))
```

Nuevamente, existen metodos que hacen exactamente lo que queremos. Este codigo
crea un array de "Boxes" y luego usa el type `T` para convertirlo a un vector

La primer parte `<[_]>` es una manera inusual de escribir "un slice de algo",
dejando que Rust infiera el type(de los elementos) por nosotros
La repeticion en este ejemplo viene cuando hacemos `$($x),*` esta es la misma
sintaxis que vimos en el patron de la tabla anterior. Esta itera sobre la lista
de expresiones que matcheamos para `$x` y las inserta a todas dentro del
template, separadas por comas

En este caso, el patron repetido de salida se ve parecido a la entrada, pero
no tiene porque ser asi. Podriamos escribir las reglas de la siguiente manera:

```rust
($($x:expr),*) => {
      let mut v = Vec::new();
      $(v.push($x);)*
      v
}
```

Aca la parte del template que lee `$(v.push($x);)*` inserta una llamada a
`v.push()` para cada expresion en `$x`.

A diferencia de el resto de Rust los patrones que usan `$(...),*` no soportan
automaticamente un coma opcional al final. Sin embargo existe un truco estandar
para hacer que las comas al final. Esto es lo que la tercer regla de nuestra
macro `vec!` hace:

```rust
($($x:expr),+,) => { // si la coma al final esta presente
   vec![$($x),*]     // vuelve a intentar sin ella
}
```
Usamos `$(...),+,` para matchear una lista que tiene una coma de mas. Entonces
en el template, llamamos recursivemente a `vec!`, dejando la coma que esta de
mas afuera. Esta vez la segunda regla debe matchear


## Macros que estan en la libreria estandar

El compilador de Rust nos da una lista de macros que pueden ser utiles cuando
estamos definiendo nuestras propias macros. Ninguna de ellas puede implementarse
usando una llamada a `macro_rules!` ya que son implementadas en el compilador
mismo `rustc`

 - `file!`: expande a un literal de string el nombre del actual archivo. Tambien
 estan `lines!` y `column!` que expande a literales `u32` la linea y la columna
 de una dado archivo. Si un macro llama a otro, el cual llama a otro todos en
 diferentes archivos y el ultimo archivo llama a `file!()`, `line!()` o
 `column!()` este expandira a la locacion indicada de la primera macro que llamo

 - `stingify!(...tokens...)`: expande a un literal de string conteniendo el dado
 `tokens`. El macro `assert!` usa esto para generar un mensaje de error que
 incluye el codigo de lo que quisimos verifica. La llamada a la macro no se
 expande o sea que si llamamo por ejemplo `stingify!(line!())` se expande solo
 el string `line!()`. Rust construye el string desde tokens, entonces no hay
 quiebres de lineas o comentarios en el string

 - `concat!(str0, str1, ...)`: expande a un solo literal de string hecho con
 la concatenacion de sus argumentos

Rust tambien tiene macros para trabajar con el ambiente de compilacion:

 - `cfg!()`: expande a una constante booleana que es `true` si la actual
 configuracion de compilacion matchea con la condicion que le ponemos en el
 parentesis. Por ejemplo: `cfg!(debug_assertions)` es `true` si estamos compilando
 con `debug_assertions` habilitadas. Esta macro soporta las mismas syntax que
 el atributo `#[cfg(...)]` pero en lugar de ser un "flag" de compilacion condicional
 lo que obtenemos es un booleano
 - `env!("VAR_NAME")`: expande un string a el valor de una variable especifica
 del entorno en tiempo de compilacion. Si la variable no existe es un error de
 compilacion. Esto puede parecer que no tiene ningun valor exepto que `cargo`
 setea muchas variables interesantes cuando compila un crate. Por ejemplo, para
 obtener la version de nuestro crate que estamos haciendo en un string, podemos
 escribir: `let version = env("CARGO_PKG_VERSION");`
 Podemos ver una lista completa de las variables de entorno en la documentacion
 de cargo:

 [cargo docs](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates)

 - `option_env!("VAR_NAME")`: es lo mismo que `env!` exepto que retorna un
 `Option<&'static str>` que es `None` si la variable especificada no esta
 seteada

Hay tres mas macros de la libreria estandar que nos dejan "traer" codigo que
esta en otros archivos al que estamos

 - `include!("file.rs")`: expande el contenido de un archivo especifico, el cual
 debe contener codigo Rust valido

 - `include_str!("file_txt")`: expande a un `&'static str` que contiene el texto
 que esta en el archivo que le pasamos. Si el archivo no existe o no tiene UTF-8
 validos tira un error de compilacion

 - `include_bytes!("files.dat")`: es lo mismo pero nada mas lo que cambia que
 el archivo es tratado como datos binarios, no como texto UTF-8. El resultado
 es un `&'static [u8]`


## El ejemplo de un macro para construir JSONs

En este ejemplo vamos a construir una macro que sirve para construir objetos
json. En el cap:10 vimos el siguiente `enum` que representaba los datos de un
objeto json:

```rust
enum Json {
   Null,
   Boolean(bool),
   Number(f64),
   String(String),
   Array(Vec<Json>),
   Object(Box<HashMap<String, Json>>)
}
```
Y como vimos la sintaxis para escribir un valor Json como salida es un poco
engorrosa:

```rust
let student = Json::Array(vec![Json::Object(Box::new(vec![
                        ("name".to_string(), Json::String("Martin Noblia".to_string())),
                        ("class_of".to_string(), Json::Number(1982.0)),
                        ("major".to_string(), Json::String("laslsal".to_string()))
                        ].into_iter().collect())),
                        Json::Object(Box::new(vec![
                        ("name".to_string(), Json::String("Juan Perez".to_string())),
                        ("class_of".to_string(), Json::Number(1982.0)),
                        ("major".to_string(), Json::String("piola".to_string()))
                        ].into_iter().collect()))
                        ]);
```
Lo que queremos es poder construir los Json como se hace habitualmente:

```rust
let student = json!([{
      "name":"Martin Noblia",
      "class_of":1982,
      "major":"lalsals"},
   {
      "name":"Juan Perez",
      "class_of": 1982,
      "major":"piola"
   }
   ]);
```

### Fragmentos de types

El primer trabajo en escribir cualquier macro compleja es ver como matchear, o
parsear la entrada deseada

Podemos anticipar que el macro va a tener muchas reglas, porque hay diferentes
tipos de cosas en un Json(numberos, strings, arrays,...etc), de hecho podemos
pensar que tendremos que hacer una regla para cada tipo de Json:

```rust
macro_rules! json {
   (null) => {Json::Null};
   ([...])=> {Json::Array};
   ([...]) => {Json::Object()};
   (???)  => {Json::Boolean(...)};
   (???)  => {Json::Number(...)};
   (???)  => {Json::String(...)};
}
```
Esto no es corrrecto ya que el patron de las macros no ofrece una manera de
separar los ultimos tres casos, pero vamos a ver como hacemos para lograrlo luego
Los primeros tres casos, al menos claramente comienzan con diferentes tokens,
comencemos con esos.

La primera regla funciona realmente(es trivial):

```rust
macro_rules! json {
   (null) => {Json::Null}
}
```

Para dar soporte a los arrays de Jsons tenemos que tratar de matchear los
elementos como exprs:

```rust
macro_rules! json {
   (null) => {
      Json::Null
   };
   ([$($element:expr),*]) => {
      Json::Array(vec![$($element),*])
   }
}
```

Ya que el patron `$($element:expr),*` significa: "una lista de expresiones validas
de Rust separadas por una coma". Pero muchos valores Jsons no son expresiones
validas para Rust.
Dado que no todo pedazo de codigo que queremos matchear es una expresion, Rust
soporta muchos otros pedazos de types(ver tabla pag:819)

Muchas de las opciones de la tabla fuerzan estrictamente a a la sintaxis de Rust
La el type de `expr` matchea solamente con una expresion de Rust(no con un valor
de Json) `ty` matchea types de Rust y asi sucesivamente. Los dos ultimos `ident`
y `tt` soportan matchear argumentos de macros que no lucen como codigo de Rust
`ident` matchea cualquier identificador, `tt` matchea un arbol de "token", como
por ejemplo un par de brakets () [] {} y cualquier cosa entre ellos, incluyendo
arboles anidados de tokens o un simple token que no es un bracket, como "1929" o
"Knots". Estos tokens son exactamente lo que necesitamos para nuestro `json!`.
Todos los valores Jsons es un simple arbol de token: numeros, strings, booleanos
y null son tokens simples; los objetos y los arrays llevan brackets. Por eso
podemos escribir patrones como el siguiente:

```rust
macro_rules! json {
   (null) => {
      Json::Null
   };
   ([$($element:tt),*]) => {
      Json::Array(...)
   };
   ({$($key:tt : $value:tt),*}) => {
      Json::Object(...)
   };
   ($other:tt) => {
      ...//TODO:
   }
}
```

Con esta version mejorada de nuestra macro podemos matchear a todos los datos
de un Json. Ahora solo nos falta producir codigo Rust que sea correcto.

Para asegurarse de que Rust pueda ganar nueva sintaxis en el futuro sin romper
ninguna de las reglas anteriore, Rust restringe los tokens que aparecen en


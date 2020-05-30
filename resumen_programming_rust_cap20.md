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
   ($($x:expr), *) => {
      <_>::into_vec(Box::new([$($x), *]))
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



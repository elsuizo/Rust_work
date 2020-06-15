#Resumen del libro \"Programming Rust\"

Rust es un lenguaje de programacion relativamente nuevo desarrollado por
Mozilla y una comunidad de contribuidores. Como `C` o `C++` Rust le da a
los desarrolladores la posibilidad de manejar la memoria con un control
total. La principal caracteristica de Rust es su novedoso sistema de
ropiedad de memoria(**ownership**)

## Cap9: \"Structs\"

## Cap: 17 String y Text

### Unicode, ASCII y latin-1

Unicode y ASCII matchean para todos los ASCII(o sea que UNICODE es un
subconjunto de ASCII) desde 0 a 0x7f: por ejemplo, los dos asignan `*` a
el codigo `42`. De manera similar Unicode asigna 0 hasta 0xff a los
mismos caracteres como un subconjunto ISO/IEC. Unicode llama a este
subconjunto Latin-1. Dado que Unicode es un superconjunto de Latin-1,
convertir Latin-1 a Unicode no requiere del uso de ninguna tabla:

``` {.rust}
fn latin_to_char(latin1: u8) -> char {
      latin1 as char
}
```

La conversion inversa es tambien una papa, asumiendo que la entrada esta
en el rango de Latin-1:

``` {.rust}
fn char_to_latin1(c: char) -> Option<u8> {
      if c as u32 <= 0xff {
            Some(c as u8)
      } else {
            None
      }
}
```

### UTF-8

Los types `String` y `str` representan texto usando la codificacion
UTF-8. UTF-8 codifica un caracter como una secuencia de 1 a 4 bytes. Hay
dos restricciones para consirerar a una secuencia UTF-8 bien formada.
Primero, solo las codificaciones que son las mas cortas posibles son las
validas para un dado punto. Esto garantiza que hay solo una codificacion
posible para un dado punto. Segundo una secuencia bien formada UTF-8
debe no codificar numeros desde 0xd800 hasta 0xdfff o mas alla de
0x10ffff: ya que estos estan reservados para cosas que no son
caracteres, o fuera del rango de los Unicodes

### Caracteres `char`

Un `char` en Rust es un valor de 32-bits que guarda un punto Unicode. Se
garantiza que cae dentro del rango desde 0 hasta 0xd7ff, o en el rango
desde 0xe000 a 0x10ffff y todos los metodos para crear y manipular
`char~s se
aseguran que esto sea cierto. El type ~char` implementa `Copy` y
`Clone`, con los traits comunes para comparacion, hashing y formato

### Manejando digitos

Para manejar digitos podemos usar los siguientes metodos:

-   `ch.to_digit(radix)`: decide cuando `ch` es un digito en la base
    `radix`, si lo es retorna `Some(num)`, donde `num` es un `u32`. De
    otra manera retorna un `None`. El parametro `radix` podemos utilizar
    el rango \[2:36\]
-   La funcion libre `std::char::from_digit(num, radix)`: convierte el
    numero `u32` en un `char` si es posible, osea que retorna un
    `Some(ch)` o `None`
-   `ch.is_digit(radix)`: retorna un `true` si el `ch` es un ASCII en
    base radix. Esto es equivalente a `ch.to_digit(radix) != None`

### `String` y `str`

Estos types garaztizan que contienen solo UTF-8 validos como elementos.
Rust pone los metodos de manejo de texto sobre `String~s o sobre ~str`
dependiendo si el metodo necesita un buffer que cambie de tamanio o esta
bien solo usar el texto sin modificar su tamanio. Dado que `String` se
desreferencia a `&str`, cada metodo definido sobre `str` esta
directamente disponible sobre `String`. El `String` es implementado como
un \"wrapper\" alrededor de `Vec<u8>` que asegura que el contenido del
vector sea siempre un UTF-8 bien formado, por eso podemos inferir que
los `String`s en Rust tienen la misma
performance que los `Vec<T>`

### Creando valores que sean `String`s

Hay algunas pocas maneras de crear valores `String`:

-   `String::new()`: retorna un nuevo `String` vacio. Todavia no tiene
    allocada memoria, pero cuando la necesite sera allocada en el heap
-   `String::with_capacity(n)`: Retorna un `String` vacio con un buffer
    pre-allocado que puede contener `n` bytes
-   `slice.to_string()`: Alloca un nuevo `String` cuyo contenido es una
    copia del slice
-   `iter.collect()`: Construye un `String` por concatenacion de los
    items de un iterador, los cuales pueden ser valores `char`, `&str`
    o `String`

### Inspecciones simples del texto

Los siguientes metodos nos dan la informacion basica desde slices de
`String`s:

-   `slice.len()`: nos da el length del slice, en bytes!!!
-   `slice.is_empty()`: si es `true` entonces `slice.len() == 0`
-   `slice[range]`: Retorna un slice que comparte la porcion dada del
    slice. Los ranges que no tienen limites son aceptados

``` {.rust}
let full = "bookkeeping";
assert_eq!(&full[..4], "book");
assert_eq!(&full[5..], "eeping");
assert_eq!(&full[2..4], "ok");
assert_eq!(&full.len(), 11); // recordar que son bytes!!!
assert_eq!(&full[5..].contains("boo"), false);
```

-   No podemos indexar un slice de string (&str, o cualquiera que
    desreferencie a `String`), como `slice[i]`, pero podemos producir un
    iterador de `char`s sobre este slice y preguntar si esta el UTF-8
    que queremos
-   `slice.split_at(i)`: retorna una tupla de dos slices que son
    compartidas por el slice, osea lo mismo que:
    `(slice[..i], slice[i..])`
-   `slice.is_char_boundary(i)`: Es `true` si el offset byte `i` cae
    entre los limites caracteres y asi es adecuado como un offset para
    un slice

Naturalmente los slices se pueden comparar por igualdad, orden y
\"hasheando\". La comparacion por orden simplemente trata a los strings
como una secuencia de puntos Unicode y los compara lexicograficamente

### Anexando e insertando texto

Los siguientes metodos anexan texto a un `String`:

-   `string.push(ch)`: anexa el caracter al final del string
-   `string.push_str(slice)`: anexa todo el contenido de un slice
-   `string.extend(iter)`: anexa los items producidos por un iterator de
    slice al string. El iterador puede producir elementos `char`, `str`
    o `String`. Estas son las implementaciones de `std::iter::Extend;`

```rust
let mut also_spaceless = "con".to_string();
also_spaceless.extend("tri but ion".split_whitespace());
assert_eq!(also_spaceless, "contribution");
```

-   `string.insert(i, ch)`: inserta el character `ch` como un offset `i`
    en el string. No es una manera recomendable de construir un string
    porque requiere tiempo cuadratico para hacerlo
-   `string.insert_str(i, slice)`: hace lo mismo para slices, con la
    misma penalidad en el performance

Strings implementan `std::fmt::Write`, queriendo decir esto que los
macros `write!()` y `writeln()` pueden anexar texto formateado a un
`String`

```rust
use std::fmt::Write;

let mut letter = String::new();
writeln!(letter, "Whose {} these are i think i known", "rutabagas")?;
writeln!(letter, "His house is in the village though;)?;
assert_eq!(letter, "Whose rutabagas these are i think i known\n\His house is
in the village though;\n");
```

Como `String` implementa `Add<&str>` y `AddAssign<&str>` podemos
escribir codigo como este:

``` {.rust}
let left = "partners".to_string();
let mut right = "crime".to_string();
assert_eq!(left + "in" + &right, "partners in crime");

right += "doesn't pay"
assert_eq!(right, "crime doesn't pay");
```

### Removiendo texto

`String` tiene unos pocos metodos para remover texto (estos no afectan
la capacidad del string, hay que usar `shrink_to_fit()` si quermos
liberar memoria)

-   `string.clear()`: reset al string para convertirlo en un `String`
    vacio
-   `string.truncate(n)`: descarta todos los caracteres despues de `n`
    bytes, dejando el string de un length de al menos `n`. Si el string
    es mas chico que `n`, esto no tiene efecto
-   `string.pop()`: remueve el ultimo caracter de un string, si es que
    hay alguno y retorna un `Option<char>`
-   `string.remove(i)`: remueve el caracter el la posicion de offset `i`
    de un string y lo devuelve, corriendo cualquier caracter que este en
    el frente de este. Esto toma un tiempo lineal en el numero de
    caracteres que tiene que mover
-   `string.drain(range)`: retorna un iterador sobre el dado rango de
    indices(en bytes) y remueve los caracteres una vez que el itarador
    es tirado. Los caracteres que quedan se mueven para el frente del
    string

### Convesiones para cuando buscamos y iteramos sobre texto

La libreria estandar de Rust para buscar texto e iterar sobre un texto
siguen algunas convensiones para hacerlas mas facil de recordar:

-   Muchas operaciones procesan texto desde el comienzo hasta el final,
    pero operaciones con nombres que comienzan con `r` trabajan desde el
    final hastal el comienzo. Por ejemplo `rsplit` es la version desde
    el final al comienzo de `split`. En algunos casos cambiar la
    direccion puede afectar so solo el orden en los que los valores son
    producidos sino que tambien a los valores en si mismo.
-   Los iteradores con nombres que terminan con `n` se limitan a si
    mismos a un numero dado de matches
-   Los iteradores con nombres que terminan con `_indices` producen
    junto con el valor usual de iteracion, los bytes de offset en el
    slice en el que aparecen

### Patrones para buscar en textos

Cuando una funcion de la libreria necesita buscar, matchear, splitear o
trimear texto, esta acepta muchos diferentes parametros para representar
lo que quiere hacer:

```rust
let haystack = "One fine day, in the middle of the night";
assert_eq!(haystack.find(','), Some(12));
assert_eq!(haystack.find("night"), Some(35));
assert_eq!(haystack.find(char::is_whitespace), Some(3));
```

Estos types los llamamos patrones y muchas operaciones las soportan:

```rust
assert_eq!("## Elephants".trim_left_matches(|ch: char| ch == '#' ||
ch.is_whitespace(), "Elephants");
```

La libreria estandar soporta cuatro principales patrones:

-   Un `char` como un patron que matchea ese character
-   Un `String` o un `&str` o `&&str` como un parametro de busqueda de
    una substring igual al patron
-   Una `FnMut(char)->bool` como un patron que matchea a un caracter
    para el cual el dado closure es `true`
-   Un `&[char]` como un patron (no un `&str` sino un slice de valores
    `char`) que matchean cualquier simple caracter que aparece en la
    lista. Notemos que si escribimos la lista como un array de
    literales, necesitamos usar una expresion para obtener el type que
    corresponde

``` {.rust}
let code = "\t function noodle() {";
assert_eq!(code.trim_left_matches(&['', '\t'] as &[char]), "function
noddle(){");
```

### Buscando y reemplazando texto

Rust tiene unos metodos para buscar en texto por patrones y posiblemente
reemplazarlo con otro texto:

-   `slice.contains(pattern)`: retorna `true` si el slice contiene un
    patron con el cual matchea
-   `slice.starts_with(pattern)` y `slice.ends_with(pattern)`: retorna
    `true` si el slice comienza o termina respectivamente con el patron
    dado por `pattern`

``` {.rust}
assert_eq!("2018".starts_with(char::is_numeric), true);
```

-   `slice.find(pattern)` y `slice.rfind(pattern)`: retorna `Some(i)` si
    el slice contiene un patron con el cual matchea con `pattern`, donde
    `i` es el offset en bytes en el que que aparece el patron. `find()`
    retorna el primer match y `rfind()` retorna el ultimo

``` {.rust}
let quip = "We also know there are known unknowns";
assert_eq!(quip.find("know"), Some(8));
assert_eq!(quip.rfind("know"), Some(31));
assert_eq!(quip.find("yay know"), None);
assert_eq!(quip.rfind(char::is_uppercase), Some(0));
```

-   `slice.replace(pattern, replacement)`: retorna un nuevo `String`
    formado por reemplazar todos los matches con `pattern` con
    `replacement`

``` {.rust}
assert_eq!("The only thing we have to fear is fear
itself".replace("fear","sping"), "The only thing we have to sping is
sping itself");
```

-   `slice.replacen(pattern, replacement, n)`: hace lo mismo pero
    reemplaza las primeras `n` matches

### Iterando sobre Texto

La libreria estandar provee muchas maneras de iterar sobre un slice de
texto. Para algunos patrones, trabajar desde el final al comienzo puede
cambiar los valores que produce(porque se topa primero con el patron o
no)

-   `slice.chars()`: retorna un iterador un slice de caracteres
-   `slice.char_indices()`: retorna un iterador sobre un slice de
    caracteres y sus offset de bytes. Notemos que esto no es lo mismo
    que `chars().enumerate()` ya que nos da el offset que tenemos que
    aplicar para saltar de un char a otro
-   `slice.bytes()`: retorna un iterador sobre los bytes individuales
    del slice exponiendo los UTF-8
-   `slice.lines()`: retorna un iterador sobre las lineas de un slice.
    Lineas se toman a lo que esta encerrado entre \"`\n`{=latex}\" o
    \"`\n`{=latex}\". Cada item produce un nuevo `&str` compartido desde
    el slice. Los items no incluyen los caracteres que terminan las
    lineas
-   `slice.split(pattern)`: retorna un iterador sobre las porciones de
    un slice separado por los patrones que matchea. Esto produce strings
    vacios entre matches adyacentes, como tambien para matches en el
    comienzo o en el final del slice
-   `slice.split_terminator(pattern)` y
    `slice.rsplit_terminator(pattern)`: son similares, exepto que el
    patron es tratado como un terminador, no como un separador, si el
    patron matchea a derecha y al final del slice, el iterador no
    produce un slice vacio representando al string vacio entre el match
    y el final del slice, como `split()` y `rsplit()` hacen. Por
    ejemplo:

``` {.rust}
// the ':' characters are separators here. Note the final ""
assert_eq!("jimb:1000:Jim Blandy:".split(':').collect::<Vec<_>>(),
vec!["jim", "1000", "Jim Blandy", ""]);
```

-   `slice.splitn(n, pattern)` y `slice.rsplit(n, pattern)` son como
    `split` y `rsplit` exepto que ellos splitean el string en al menos
    `n` slices y los primeros `n-1` matchean con el patron
-   `slice.split_whitespace()`: retorna un iterador sobre las porciones
    del slice que estan separadas por un espacio en blanco, cuando
    tenemos una separacion de varios espacios en blanco se toma como
    uno, los espacios en blanco al final del slice se ignoran. Esto usa
    la misma definicion de espacio en blanco que `char::is_whitespace`

### Cortando los slices

Cuando queremos cortar partes de un string es usualmente cuando queremos
sacarle los espacios en blanco del comienzo o en el final del string,
tambien cuando queremos limpiar una entrada que leemos de un file

-   `slice.trim()`: retorna un subslice del slice original que omite
    cualquier espacio en blanco al comienzo o al final.
    `slice.trim_left()` omite los espacios en blanco al comienzo y
    `slice.trim_right()` omite los espacios en blanco al final
-   `slice.trim_matches(pattern)`: retorna un subslice que omite todos
    los matches de un patron desde el comienzo al final, lo mismo con
    `trim_left_matches()` y `trim_right_matches()` que hacen lo mismo
    con los patrones que estan al comienzo y al final del slice

``` {.rust}
assert_eq!("0001990".trim_left_matches('0'), "1990");
```

### Parseando otros types desde un `String`

Rust provee traits standards para parsear valores desde un string y
producir representaciones textuales de valores

Si un type implementa el trait `std::str::FromStr`, entonces este provee
una manera estandar de parsear un valor desde un slice (por ejemplo:
`&str`)

``` {.rust}
pub trait FromStr: Sized {
      type Err;
      fn from_str(s: &str) -> Result<Self, Self::Err>;
}
```

Todos los types mas comunes impl este trait

``` {.rust}
use std::str::FromStr;

assert_eq!(usize::from_str("234234"), Ok(234234));
assert_eq!(f64::from_str("1.234234"), Ok(1.234234));
assert_eq!(bool::from_str("true"), Ok(true));
```

### Convirtiendo otros types a `String`

Hay tres maneras principales de convertir valores no textuales en
`String`s

-   Types que tienen una manera natural que es humanamente leible puede
    implementarse con el trait `std::Display`, el cual nos deja usar los
    brackets {} para formatear en el macro `format!()`

``` {.rust}
assert_eq!(format!("{}, wow", "doge"), "doge, wow");
assert_eq!(format!("{}", true), "true");
assert_eq!(format!("({:.3}, {:.3})", 0.5, f64::sqrt(3.0)/2.0), "(0.500,
0.866)");
```

Si un type implementa `Display`, la libreria estandar automaticamente
implementa el trait `std::str::ToString` para el, el cual es muchas
veces los suficiente si no necesitamos la flexibilidad de `format!`

Para nuestros propios types debemos generalmente impl `Display` en lugar
de `ToString`, porque es menos flexible

### Formateando valores

Vimos que podemos formatear usando los macros que existen para formatear
como `println!`, donde el string literal sirve como template para el
output de cada `{..}` que sera reemplazado por el argumento respectivo.
Muchas librerias comparten estas convenciones para formatear strings:

-   El macro `format!` se usa para construir \~String\~s
-   Los macros `println!` y `print!` escriben texto formateado a el
    stream de salida
-   Los macros `writeln!` y `write!` escriben a una salida que le
    digamos
-   El macro `panic` se usa para poder explicar cuando un error fatal ha
    ocurrido

Podemos extender estas macros para que soporten nuestros propios types
implmentando los traits del modulo `std::fmt`.

Los templates que ponemos dentro de los literales `{}` se llaman
parametros de formato y tienen la forma: `{cual:como}`. Cualquiera de
las dos partes son opcionales. El valor de `cual` selecciona cual
argumento seguido del template debe tomar el lugar del parametro,
podemos seleccionar los argumentos por indice o por nombre. Los
parametros que no tienen valor son simplemente emparejados con los
argumentos desde izquierda a derecha. El argumento `como` nos dice como
el argumento debe ser formateado

### Formateando valores de texto

Cuando formateamos un valor textual como `&str` o `String` (un `char` es
tratado como un string de un solo caracter), el valor de `como` tiene
muchas partes, que son todas opcionales:

-   Un limite para el largo del texto. Rust trunca el argumento si es
    mas largo que este limite, si no especificamos nada usa todo el
    texto
-   Un ancho minimo. Despues de cualquier truncado si el argumento es
    mas chico que esto, Rust lo padea a la derecha(por default) con
    espacios(por default) para hacer que el field tenga ese ancho
-   Una alineacion. Si tu argumento necesita ser padeado para cumplir
    con un minimo ancho, esto le dice como tiene que ser puesto si a la
    comienzo(\<), al centro (\^) o al final(\>)
-   Un caracter de padding para usar en el proceso de padeo. Si se omite
    Rust usa espacios. Si especificamos el caracter de padding debemos
    tambien especificar la alineacion

Ver tabla pag: 666

### Formateando numeros

Cuando el argumento a formatear tiene un type numerico como `usize` o
`f64`, los parametros de `como` tienen las siguientes partes todas
opcionales:

-   Un argumento de **pading** y de **alineamiento**, el cual trabaja de
    la misma manera que lo hace con los types textuales
-   Un caracter `+`, con el que pedimos que se muestre siempre el signo
    del numero que estamos imprimiendo, aun cuando el numero es positivo
-   Un caracter `#`, con el que pedimos que nos muestre un radix
    explicito como `0x` o `0b`
-   Un caracter `0`, con el que pedimos que se rellene con ceros el
    ancho total del campo, en lugar de usar pading
-   Un ancho minimo para el campo. Si el numero que queremos formatear
    no tiene al menos este ancho, Rust rellena este con espacios(por
    default) a la izquierda(por default) para hacer al campo que tenga
    el ancho dado.
-   Una precision para los numeros de punto flotante, indicando cuantos
    digitos queremos que Rust incluya despues del punto decimal. Rust
    redondea o rellena con ceros para producir exactamente el numero de
    decimales que queremos. Si se omite, Rust trata de ser lo mas
    preciso posible representando el valor con la menor cantidad de
    digitos.
-   Una notacion. Para enteros, esto puede ser `b` para binarios o para
    octales `x` o para hexadecimales `X` con mayuscula o minuscula. Si
    incluimos el caracter `#`, esto incluye un estilo de Rust explicito
    para el prefijo radix, `0b`, `0o`, `0x` o `0X`. Para punto flotante
    un radix de `e` o `E` pide notacion cientifica, con un coeficiente
    normalizado, usando `e` o `E` para el exponente. Si no especificamos
    ninguna notacion, Rust formatea el numero en decimal.

Ver tabla de la pag: 670, para ejemplos

### Formateando nuetros propios types

Los macros que formatean usan un conjunto de traits definidos en el
modulo `std::fmt` para convertir valores a texto. Podemos hacer que
estos macros funcionen para nuestros types implementando uno o varios de
estos traits. La notacion de un parametro de formateo indica cual trait
debe implementar el argumento. Ver tabla pag: 677

Como sabemos podemos hacer que derive un trait para nuestros types con
`#[derive(Debug)]`, con ello podremos usar el formateador `{:?}`

Los traits de formateo siguen todos la misma estructura, solo difieren
en los nombres, como ejemplo ponemos a `std::fmt::Display`

``` {.rust}
trait Display {
      fn fmt(&self, dest: &mut std::fmt::Formatter) -> std::fmt::Result;
}
```

El proposito de el metodo `fmt` es producir una representacion
propiamente formateada de `self` y escribir estas caracteres a `dest`

Por ejemplo tenemos un type que representa los numeros complejos

``` {.rust}
use std::fmt;

impl fmt::Display for Complex {
      fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
            let i_sign = if self.i < 0.0 {'-'} else {'+'};
            write!(dest, "{}{}{}i", self.r, i_sign, f64::abs(self.i))
      }
}
```

### Usando el lenguaje de formato en nuestro propio codigo

Podemos escribir nuestras propias funciones y macros que acepten
templates de formato y argumentos usando el marco de Rust
`format_args!()` y el type `std::fmt::Arguments`. Por ejemplo supongamos
que nuestro programa necesita logear un mensaje de estatus mientras esta
corriendo y queremos usar los formateadores de texto de Rust para
producir ese log. Por ejemplo podemos hacer:

``` {.rust}
fn logging_enabled() -> bool {
      //...
}

use std::fs::OpenOptions;
use std::io::Write;

fn write_log_entry(entry: std::fmt::Arguments) {
      if logging_enabled() {
            // keeps things simple for now, and just open the file every
            // time
            let mut log_file =
            OpenOptions::new()
            .append(true)
            .create(true)
            .open("log-file-name")
            .expect("failed to open log file");

            log_file.write_fmt(entry).expect("failed to write to log");
      }
}
```

Luego podemos llamar a la funcion asi:

`write_log_entry(format_args!("Hark!{:?}\n", value))`

En tiempo de compilacion, la macro `format_args!` parsea el string que
pusimos como template y chequea sus argumentos contra los types,
reportando un error si hay un problema

Pero esta llamada a la funcino `write_log_entry()` no es muy bonita.
Podemos escribir un macro para que parezca mas presentable:

``` {.rust}
macro_rules! log {// no necesitamos poner el ! en la definicion de una macro
      ($format:tt, $($arg:expr), *) => (
            write_log_entry(format_args!($format, $($arg),*))
      )
}
```

Vamos a ver macros mas adelante en el cap: 20

Ahora podemos llamar a nuestra nueva macro asi:

`log!("The telemetry values are: {:?}", telemetry);`

### Expresiones regulares

El crate externo a la libreria estandar \"regex\" es el crate cuasi
oficial para trabajar con expresiones regulares en Rust. Provee las
funciones basicas de busqueda y matcheo, tiene un buen soporte para
Unicode, pero puede tambien buscar strings de bytes

1.  Uso basico de expresiones regulares

    Un valor de \"Regex\" representa un valor parseado, listo para ser
    usado. El constructor de un \"Regex\" intenta parsear un `&str` como
    una expresion regular y retorna un `Result`

    ``` {.rust}
    use regex::Regex;
    // usamos r"..." para que lo tome como un string raw
    let semver = Regex::new(r"(\d+)\.(\d+)\.(\d+)(-[-.[:alnum:]]*)?")?;
    // hacemos una simple busqueda que nos da un booleano de resultado
    let haystack  = r#"regex = "0.2.5"#";
    assert_eq!(semver.is_match(haystack));
    ```

    El metodo `Regex::captures()` busca en un string por el primer match
    y retorna un valor del type `Regex::Catures` conteniendo la
    informacion de los matches que ha encontrado

    ``` {.rust}
    // podemos sacar esos valores que capturamos
    let captures = semver.captures(haystack).ok_or("semver regex should have
    mathced")?;

    assert_eq!(&captures[0], "0.2.5");
    assert_eq!(&captures[1], "0");
    assert_eq!(&captures[2], "2")
    assert_eq!(&captures[3], "5")
    ```

    Podemos iterar sobre todos los matches en un string:

    ``` {.rust}
    let haystack = "in the beginning, there was 1.0.0.\
    for a while, we used 1.0.1-beta,\
    but in the end, we settled on 1.2.4.";

    let matches: Vec<&str> = semver.find_iter(haystack).map(|m|
    m.as_str()).collect();

    assert_eq!(matches, vec!["1.0.0", "1.0.1-beta", "1.2.4"]);
    ```

2.  Creando regex que son un poco vagos...

    El constructor `Regex::new()` puede ser costoso computacionalmente
    para una expresion regular de 1200 caracteres, que puede tomar casi
    un milisegundo sobre una maquina rapida. Es mejor para estos casos
    sacar a la construccion de este regex del loop, podriamos en cambio
    construirlo una vez y usarlo tantas veces queramos. El crate
    `lazy_static` provee una manera conveniente de contruir valores que
    no se ejecutan hasta cuando son llamados, este crate provee un macro
    para declarar estas variables:

    ``` {.rust}
    #[macro_use]
    extern crate lazy_static;

    lazy_static! {
          static ref SEMVER: Regex =
          Regex::new(r"(\d+)\.(\d+)\.(\d+)(-[-.[:alnum:]]*)?").expect("error
          parsing the regex");
    }
    ```

    El macro se expande a una declaracion de una variable llamada
    `SEMVER` pero su type no es exactamente `Regex`, en cambio es un
    type generado por un macro que implementa `Deref<Target=Regex>` y
    por ello expone todos los mismos metodos que un `Regex`

## Cap: 18. Input y Output

Las librerias estandar de Rust sobre entrada-salida estan organizadas
alrededor de tres traits, `Read`, `BufRead`, `Write` y de los varios
types que los implementan:

-   Valores que implementan `Read` tienen metodos para inputs que se
    orientan a bytes. Los llamamos \"readers\"
-   Valores que implementan `BufRead` son llamados \"buffered readers\".
    Ellos soportan todos los metodos de `Read`, mas metodos para leer
    lineas de texto
-   Valores que implementan `Write` soportan ambos orientados a bytes y
    UTF-8 como salidas. Los llamamos \"writers\"

Podemos resumir como se relacionan los traits con los respectivos types
de la libreria estandar:

``` {.text}
~Read~
  +---> ~Stdin~
  |
  +---> ~File~
  |
  +---> ~TcpStream~
  |
  |     +-----------+
  +---> |           |
        | ~BufRead~ |
        |           |
        +-----------+
             |
             |
             +---> ~BufRead<R>~
             |
             |
             +---> ~Cursor<&[u8]>~
             |
             |
             +---> ~StdinLock~

```

``` {.text}
+-----------+
|           |
| ~BufRead~ |
|           |
+-----------+
      |
      |
      +---> Stdout
      |
      |
      +---> Stderr
      |
      |
      +---> File
      |
      |
      |---> TcpStream
      |
      |
      +---> Vec<u8>
      |
      |
      +---> BufWriter<W>
```

### \"Readers\" y \"Writers\"

Los \"Readers\" son valores que tu programa puede leer bytes desde.
Algunos ejemplos serian:

-   Abriendo archivos usando `std::fs::File::open(file_name)`
-   `std::net::TcpStream` para recibir datos en una red
-   `std::io::stdin()` para leer desde el stream de stdin
-   valores `std::io::Cursor<&[u8]>` los cuales son los \"readers\" que
    leen desde un array de bytes que ya estan en memoria

Los \"Writers\" son valores que tu programa puede escribir bytes a.
Algunos ejemplos son:

-   Archivos que fueron abiertos usando
    `std::fs::File::create(file_name)`
-   `std::net::TcpStream` para enviar datos en una red
-   `std::io::stdout()` y `std::io::stderr()` para escribir en la
    terminal
-   valores `std::io::Cursor<&mut [u8]>` los cuales nos dejan tratar
    cualquier slice mutable de bytes como si fuera un archivo al que
    podemos escribir
-   `Vec<u8>`, un \"writer\" cuyos metodos tienen como salida un vector

Dado que hay traits que pertenecen a la libreria estandar para
\"readers\" y \"writers\" es muy comun que podamos escribir codigo
generico que funcione para muchos types de inputs y outputs. Por ejemplo
la siguiente funcion que copia todos los bytes desde cualquier
\"reader\" a cualquier \"writer\"

Esa es la implementacion del metodo `std::io::copy()` de la libreria
estandar de Rust. Dado que es generica podemos utilizarla para copiar
datos de un `File` a un `TcpStream` o desde `Stdin` a un `Vec<u8>` en
memoria and so on... Estos cuatro traits `Read`, `Write`, `BufRead` y
`Seek` son tan comunes que son incluidos en el prelude:
`use std::io::prelude::*;`

### \"Readers\"

El `std::io::Read` tiene muchos metodos para leer datos. Todos ellos
toman el \"reader\" mismo por una referencia mutable

-   `reader.read(&mut buffer)`: Lee algunos bytes desde una fuente de
    datos y los guarda en el `buffer` que le pasamos. El type para el
    `buffer` debe ser `&mut [u8]`. Y se leen bytes hasta que llenamos el
    `buffer` o sea hasta `buffer.len()` El type de retorno es un
    `io::Result<u64>`, el cual es un type alias para
    `Result<u64, io::Error>`. Cuando leemos correctamente el valor de
    `u64` es el numero de bytes que leimos(el cual puede ser igual o
    menor que los bytes que tiene el buffer (`buffer.len()`)), aun si
    hay mas datos por venir. `Ok(0)` significa que no hay mas entradas
    para leer. Si hubo un error, `read()` retorna un `Err(err)` donde
    `err` es un valor `io::Error`. Un error de este type es imprimible,
    para que podamos ver que paso, por eso tiene un metodo llamado
    `.kind()` que retorna un type de codigo como `io::ErrorKind`. Los
    miembros de este `enum` tienen nombres como `PermissionDenied` o
    `ConnectionReset` que son errores lo sufientemente serio para ser
    ignorados. Como podemos ver `read()` es lo sufientemente
    \"low-level\" para tratar con cosas del SO directamente, por eso si
    queremos impl `Read` para nuestros types es recomendable solo si
    vamos a hacer cosas \"low-level\" porque si no es mucho trabajo
    hacerlo, en cambio Rust nos provee metodos que son mas
    \"high-level\" los cuales tienen implementaciones por \"default\"
    para `read()` y todos manejan el `ErrorKind::Interrupt` por
    nosotros!!!
-   `reader.read_to_end(&mut byte_vec)`: Lee toda la entrada restante
    desde el `reader` anexandolo al vector que le pasamos, el cual es
    del type `Vec<u8>`. Retorna un `io::Result<()>`
-   `reader.read_to_string(&mut string)`: es lo mismo, pero agrega los
    datos a el dado `String`. Si no es valido UTF-8, retorna un error
    `ErrorKind::InvalidData`. En algunos lenguajes entadas de bytes y
    chars son manejadas con diferentes types, como el disenio de Rust es
    predominante el uso de UTF-8 entonces no hace falta tratarlos
    distinto
-   `reader.read_exact(&mut buff)`: reads exactamente los datos
    suficientes para llenar el buffer dado. El type del argumento debe
    ser `&[u8]`. Si el reader si queda sin datos antes de que pasen los
    `buffer.len()` bytes esto retorna un error
    `ErrorKind::UnexpectedEof`

Estos son los principales metodos del trait `Read`. Ademas hay cuatro
metodos adaptadores que toman un \"reader\" por valor, lo transforman en
un iterador o en un \"reader\" diferente:

-   `reader.bytes()`: retorna un iterador sobre los bytes de una stream
    de entrada. El item que retorna a cada `next()` es un
    `io::Result<u8>` entonces un chequeo de error es requerido para cada
    byte. Ademas, esto llama a `reader.read()` una vez por byte lo cual
    es muy ineficiente si el \"reader\" no es almacenado
-   `reader.chars()`: es lo mismo pero itera sobre chars, tratando a la
    entrada como UTF-8. Si hay algun char que no es valido UTF-8 causa
    un error de `InvalidData`
-   `reader.chain(reader2)`: retorna un nuevo reader que produce todas
    las entradas para el `reader` seguidas de las entradas para el
    `reader2`
-   `reader.take(n)`: retorna un nuevo reader que lee desde la misma
    fuente que `reader` pero es limitado a `n` bytes de entrada

No hay metodos para cerrar un \"reader\" (como en otros lenguajes que
tenemos que hacerlo a mano). Los \"readers\" tipicamente implementan
`Drop`

### \"Buffered Readers\"

Por eficiencia los \"readers\" y los \"writers\" pueden ser almacenados,
o sea que tengan un pedazo de memoria (un buffer) que mantenga algunas
entradas o salidas en memoria. Esto salva llamadas al SO. La aplicacion
lee datos desde el `BufReader`, en este ejemplo llamando a su metodo
`read_line()`. Asi el `BufReader` se transforma para el SO en una
porcion de memoria larga de donde leer

Estos readers implementan tanto `Read` como un segundo trait `BufRead`
el cual tiene los siguientes metodos:

-   `reader.read_line(&mut line)`: lee una linea de texto y la pone en
    `line` que es un `String`. El caracter de nueva linea `'\n'` en el
    final de una linea es incluido en un archivo. El valor de retorno es
    un `io::Result<usize>`, los numeros de bytes leidos incluyen los
    caracteres de fin de linea. Si el \"reader\" esta en el final de la
    entrada, esto deja a la `line` sin cambios y retorna `Ok(0)`
-   `reader.line()`: retorna un iterador sobre las lineas de una
    entrada. El type del `item` que genera es un `Result<String>`. Los
    caracteres de nueva linea no se incluyen en el `String`. Este metodo
    es casi siempre lo que queremos para leer entradas de texto.
-   `reader.read_until(stop_byte, &mut byte_vec)` y
    `reader.split(stop_byte)`: son como `read_line()` y `lines()` pero
    orientado a bytes, produciendo `Vec<u8>` en lugar de `String~s.
     Nosotros elegimos el delimitador ~stop_byte`

`BufRead` tambien provee un par de metodos \"low-level\" como
`fill_buf()` y `consume(n)` para acceso directo a el buffer interno del
\"reader\"

### Leyendo lineas

Podemos implementar la utilidad de Unix `grep`. Busca en muchas lineas
de texto, tipicamente desde otro comando de Unix pipeado una dada
`String`

[implementacion](../grep-alternative/src/main.rs)

Dado que queremos llamar a `lines()`, necesitamos una entrada que
implemente `BufRead`. En este caso, llamamos a `std::io::stdin()` para
obtener los datos que son \"piped\" a nosotros. Sin embargo, la libreria
estandar de Rust proteje a `stdin` con un `Mutex` por eso llamamos a
`lock()` para \"lockear\" el recurso, al final del loop se libera el
`lock()~(sin este mutex, dos hilos
pueden estar queriendo leer el ~stdin` al mismo tiempo lo que causaria
un \"undefined behavior\"). `C` tiene el mismo problema y lo resuelve de
la misma manera pero a nivel de SO. Ya que el iterador produce valores
que son `Result` usamos el operador `?` para chequear(o en este caso
propagar hacia arriba) los errores Supongamos que queremos hacer que
nuestro programa tambien funcione buscando archivos en el disco duro.
Podemos hacerlo generico:

Notemos que `File` no es automaticamente puesto en memoria
(\"buffered\"). `File` implementa `Read` pero no `BufRead`. Sin embargo
es facil crear un buffer en memoria para `File` o cualquier otro lector
que no tenga buffer en memoria. Con `BufReader::new(reader)` hacemos
esto y tambien podemos elegir si queremos que tenga un size determinado
usando `BufReader::with_capacity(size, reader)`

En muchos lenguajes los files son buffered por default. Si no queremos
esto debemos hacerlo a mano nosotros. En Rust son dos cosas separadas.
Porque a veces queremos files sin tener buffers y a veces queremos
buffers sin tener que usar el file(por ejemplo cuando leemos un input de
una red)

### Colectando lineas

Muchos de los metodos de un reader, incluyendo `lines()` retorna un
`Iterator` que produce como valores \~Result\~s. La primera vez que
queremos colectar todas las lineas de un file en un vector grande,
podemos tener problemas en sacar los items del `Result`

`// esta bien pero no es lo que buscamos`
`let results: Vec<io::Result<String>> = reader.lines().collect();`

`// pero no podemos directamente`
`let lines: Vec<String> = reader.lines().collect();`

La solucioin poco elegante es hacer un loop

``` {.rust}
let lines = vec![];
for line in reader.lines() {
      line.push(line?);
}
```

No esta mal pero estaria bueno si podemos hacer un `collect()`
directamente..., y podemos de la siguiente manera:

`let lines = reader.lines().collect::<io::Result<Vec<String>>>()?;`

### Writers

Como vimos leer una entrada se hace casi exclusivamente con metodos
adecuados. Escribir una salida es un poco diferente. Casi siempre
utilizamos el macro `println!` para producir una salida de texto plano.
Para enviar una salida a un \"writer\", usamos los macros `write!` y
`writeln!`, que son lo mismo que `print!` y `println!` exepto por dos
diferencias:

`writeln!(io::stderr(), "error: file not found")?;`

`writeln!(&mut byte_vec, "The telemetry has size of {:?} and date: {}",`
`telemetry.size(), telemetry.date);`

Una de las diferencias es que los `write` macros toman un parametro mas
un \"writer\", la otra diferencia es que retornan un `Result`, entonces
los errores pueden ser manejados. El trait `Write` tiene los siguientes
metodos:

-   `writer.write(&buf)`: escribe algunos de los bytes en el slice que
    le pasamos. Retorna un `io::Result<usize>`. Cuando todo salio bien
    nos da el numero de bytes escritos, los cuales tienen que ser menos
    que `buf.len()`. Como `Reader::read()`, esto es el metodo mas
    \"low-level\" que tenemos que evitar usar si no lo necesitamos en
    verdad
-   `writer.write_all(&buf)`: Escribe todos los bytes en el slice `buf`
    que le pasamos, retorna un `Result<()>`
-   `writer.flush()`: Elimina cualquier dato que hayamos puesto en el
    stream. Retorna un `Result<()>`

Como con `BufReader::new(reader)` agrega un buffer a cualquier reader,
`BufWriter::new(writer)` agrega un buffer a cualquier writer

``` {.rust}
let file = File::create("tmp.txt")?;
let writer = BufWriter::new(file);
```

### Files

Ya vimos dos maneras de abrir un file:

-   `File::open(file_path)`: Abre un file para leerlo. Retorna un
    `io::Result<File>` y tira un error si el file no existe
-   `File::create(file_path)`: crea un file nuevo para ser escrito. Si
    el file existe con el nombre que le pasamos, este es truncado

Notemos que el `File` esta en el modulo de filesystems `std::fs` no en
`std::io`, si ninguna de las anteriores es suficiente podemos usar
`OpenOptions` para especificar el comportamiento deseado

``` {.rust}
use std::fs::OpenOptions;

let log = OpenOptions::new()
            .append(true) // si el file existe agregalo al final
            .open("server.log")?;

let file = OpenOptions::new()
            .write(true)
            .create_new(true) // falla si el file existe
            .open("new_file.txt")?;
```

los metodos `.append()` `.write()` `.create_new()` y asi.. son
designados para que los pongamos como si fueran una cadena, cada uno
retorna un `self`. Este metodo de encadenar metodos se llama
\"builder\". `std::process::Command` es otro ejemplo de \"builder\"

Una vez que el file fue abierto se comporta como cualquier otro `Reader`
o `Writer`

### Seeking

File tambien implementa el trait, lo que significa que podemos recorrer
un `File` en lugar de hacerlo de una sola pasada desde el principio
hasta el final `Seek` se define como sigue:

``` {.rust}
pub trait Seek {
 fn seek(&mut self, pos: SeekFrom) -> io::Result<u64>;
}

pub enum SeekFrom {
 Start(u64),
 End(i64),
 Current(i64)
}
```

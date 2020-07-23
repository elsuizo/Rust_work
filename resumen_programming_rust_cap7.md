# Cap7: Error Handling

Los errores ordinarios son manejados usando `Result<T, E>`s. Estos son cusados
generalmente por cosas que vienen desde afuera del programa en si como un input
erroneo, un error de permisos, un error de nombre en un file.
En cambio el otro tipo de error en Rust es cuando un programa "paniquea" que son
los errores que nunca deberian suceder


## Panic

Un programa "paniquea" si encuentra un error que debe ser un bug en el programa
como por ejemplo:

   - Acceder a elementos de un array fuera de sus limites(out-of-bounds)

   - Dividir por cero con enteros

   - Llamar un `.unwrap()` sobre un `Option` que tiene un `None` adentro

   - Que falle un `assert!()` o un `assert_eq!()`

Tambien podemos forzar a este comportamiento cuando queremos que nuestro
programa panique deliberadamente bajo alguna situacion. Lo podemos hacer con el
macro `panic!()` que acepta como argumento un mensaje de explicacion de porque
el programa paniquea

## Unwindig

Quizas el nombre de *panics* no es una buena eleccion ya que es un proceso que
tiene un orden y ya que no es un chrash tampoco un UB, es mas a un
`RunTimeException` en Java o un `std::logic_error` en Cpp. Este comportamiento
esta bien definido solo que no deberia suceder.
Paniquear es seguro no viola ninguna de las reglas que vimos de seguridad que
brinda Rust. Los paniqueos son por *threads* asi que si un *thread* paniquea los
demas pueden seguir haciendo lo que estaban haciendo. Existe una manera de
volver atras las variables del stack o sea recuperarnos de un paniqueo(pero en
el libro no se trata eso). Tambien el comportamiento a un paniqueo es customizable
ya que podemos desde la compilacion con `-C panic=abort` que hara que el primer
paniqueo que tenga nuestro programa inmediatamente aborte el proceso(y con esto
logramos que el size del binario sea mucho mas chico)

## Result

Rust no tiene *exceptions*, en cambio tiene funciones que puden fallar que tienen
un type de retorno que nos avisa eso:

`fn get_weather(location: Latng) -> Result<WeatherReport, io::Error>`

Como vemos el type de retorno indica un posible fallo. Cuando llamamos a la
funcion, esta puede retornar un suceso exitoso que sera un `Ok(weather)` o un
error `Err(error_value)` donde `error_value` es un `io::Error` explicando que
fue lo que fallo

## Atrapando errores

La manera mas completa de atrapar errores en Rust es como vimos en el capitulo2
usando un `match`

```rust
match get_weather(hometown) {
   Ok(report) => {display_weather(hometown, &report);}
   Err(err)   => {
      println!("error querying the weather: {}", err);
      schedule_weather_retry():
   }
}
```

Esto es mas o menos el equivalente del `try/catch` en otros lenguajes
Usar `match` esta bien pero es un poco engorroso tener que hacer esto todo el
tiempo, por ello el type `Result<T, E>` ofrece una variedad de metodos que son
utiles en casos particulares que son muy comunes. Cada uno de estos metodos tiene
bajo el capot un `match`

 - `result.is_ok()` y `result.is_err()`: Retornan un `bool` diciendonos si el
   resultado exitoso o no

 - `result.ok()`: Retorna el valor de exito, si es que hay alguno como un
   `Option<T>`. Si el valor de retorno es exitoso entonces retornara un
   `Some(succes_value)` de otra manera un `None`

 - `result.err()`: retorna el valor de error si es que ha sucedido como un
   `Option<E>`

 - `result.unwrap_or(fallback)`: retorna el valor de exito, si el resultado es
   exitoso, de otra manera retorna `fallback` descartando el valor de error!!!

 - `result.unwrap_or_else(fallback_fn)`: es lo mismo que el anterior pero en
   lugar de pasar un valor de `fallback` directamente le pasamos una funcion o
   un closure. Este es el caso donde es una perdida de tiempo tratar con un valor
   de `fallback` si no vamos a utilizarlo. Esta funcion que le pasamos se
   ejecutara solo si recibimos un `error` como `result`

 - `result.unwrap()`: tambien retorna el valor exitoso, sin embargo si el `result`
   es un `error` el programa paniquea

 - `result.expect(message)`: es lo mismo que el anterior pero por lo menos podemos
   poner un mensaje de porque fallo

 - `result.as_ref()`: convierte un `Result<T, E>` en `Result<&T, &E>` prestando
   una referencia al valor de error o de exito

 - `result.as_mut()`: es lo mismo que el anterior pero ahora presta una refrencia
   mutable, ose que el valor de retorno es: `Result<&mut T, &mut E>`

Los ultimos dos metodos son utiles ya que todos los anteriores (exepto `.is_ok()`
y `.is_err()`) consumen el resultado. Esto es que toman a `self` por valor. A
veces es util acceder a los datos que hay dentro de un `Result<T, E>` si destuirlo
y por ello es que los metodos `.as_ref()` y `as_mut()` son utiles. Por ejemplo
supongamos que queremos llamar a `result.ok()` pero necesitamos que `result`
quede intacto, podemos escribir `result.as_ref().ok()` lo que hara que comparta
el result retornando un `Option<&T>` en lugar de un `Option<T>`


## alias para el type `Result`

Algunas veces vemos en la documentacion que se omite el type del error de un
`Result` como por ejemplo:

```rust
fn remove_file(path: &Path) -> Result<()> {}
```

Esto significa que se ha usado un alias para el `Result`. Un type alias es como
un sinonimo para el nombre de un type. Algunos modulos definen su type alias
para evitar tener que repetir un type de error una y otra vez. Por ejemplo en
la libreria estandar en el modulo `std::io` se incluye la siguiente linea:

```rust
pub type Result<T> = result::Result<T, Error>;
```
Esto define un type publico `std::io::Result<T>` que es un alias para `Result<T, E>`
pero se hardcodea el `std::io::Error` como el type de error, en terminos practicos
esto quiere decir que si nosotros utilizamos `std::io` entonces Rust entiende
`io::Result<String>` como un alias para `Result<String, io::Error>`.

Cuando en un codigo vemos que una funcion o metodo retorna `Result<()>` que
aparece en la documentacion, podemos clikear sobre el identificador `Result` para
ver cual es el type alias que se ha utilizado y aprender del type de error,
usualmente es conocido de antemano dado el contexto


## Imprimiendo errores

A veces la unica manera de manejar un error es pasandolo a la terminal y seguir
Vimos que una manera de hacer esto es:

```rust
println!("error querying the weather: {}", err);
```

La liberia estandar define muchos types de errores con nombres aburridos:
`std::io::Error`, `std::fmt::Error`, `std::str::Utf8Error` y asi.... Todos ellos
implementan una interfaz comun, el trait `std::error::Error` lo que significa
que comparten las siguientes caracteristicas:

 - Todos ellos se pueden imprimir en pantalla con `println!` con el formateador
   `{}` que tipicamente imprime un mensaje breve. Alternativamente podemos
   imprimir con el famoso `{:?}` para obtener una vista de `Debug` del error

 - `err.descriptio()`: retorna un mensaje de error como un `&str`

 - `err.cause()`: retorna una `Option<&Error>` el error que esta bajo el capo
   si es que hubo alguno, que haya lanzado el mismo
   Por ejemplo cuando un error de conexion causa que una transaccion de banco
   falla, lo que puede causar que caigamos en la banca rota(jajsj). Si pusieramos
   que `err.description()` sea "boat was repossessed", entonces `err.cause()`
   puede retornar un error sobre la falla en la transaccion, su `.description()`
   deberia ser algo: "failed to transfer $300 to United Yacht Supply" y su
   `.cause()` podria ser un `io::Error` con detallles sobre el error especifico
   de conexion que ha causado que no podamos hacer la transaccion

   ```rust
   use std::error::Error;
   use std::io::{Write, stderr};

   /// Dump an errro message to `stderr`.
   ///
   /// if another error happens while building the error message or writin to
   /// `stderr`, it is ignored
   fn print_error(mut err: &Error) {
      let _ = writeln!(stderr(), "error: {}", err);
      while let Some(cause) = err.cause() {
         let _ writeln!(stderr(), "caused by: {}", cause);
         err = cause;
      }
   }
   ```
## Propagando errores

En muchos lugares donde intentamos algo que puede fallar, no queremos hacer un
"catch and handle" del error inmediatamente, es simplemente mucho codigo para
usar 10 lineas de codigo cada vez que tengamos que ver si algo anduvo mal. En
cambio, si un error ocurre, usualmente lo que queremos que el que llama a la
funcion o codigo que se encargue de ellos cuando usa el codigo. Lo que queremos
es que los errores se propaguen hacia arriba en el stack de llamadas a funciones
Rust tiene un operador para ello `?`. Podemos usar este operador `?` para cualquier
expresion que produzca un `Result` como es el resultado de el llamado a una
funcion:

```rust
let weather = get_weather(hometown)?;
```
El comportamiento de `?` depende de cuando esta funcion retorna un valor exitoso
o no.

   - Sobre un suceso, el desenvuelve el `Result` para tener acceso al valor que
     hay dentro.

   - Sobre un error, inmediatamente retorna desde donde fue llamada la funcion
     pasandole el error un nivel mas arriba de donde fue llamada la funcion.
     Para asegurar que este operador funciona solo puede ser utilizado en
     funciones que tengan un `Result` como valor de retorno

No hay magia en el operador `?` solo que expresa algo usual de una manera acotada


```rust
let weather = match get_weather(hometown) {
   Ok(success_value) => success_value,
   Err(err)          => return Err(err)
}
```

Es facil de ver en algunos programas cuantas fuentes de errores pueden haber en
un simple programa, particularmente en codigo que tiene interfaces con el SO. El
operador `?` a veces se muestra en cada linea de codigo

```rust
use std::fs;
use std::io;
use std::path:Path;

fn move_all(src: &Path, dst: &Path) -> io::Result<()> {
   for entry_result in src.read_dir()? {   // open the dir could fail
      let entry = entry_result?;           // reading dir could fail
      let dst_file = dst.join(entry.file_name());
      fs::rename(entry.path(), dst_file)?; // renaming could fail
   }

   Ok(())
}
```

## Trabajando con muchos types de errores

A menudo mas de una cosa puede salir mal. Supongamos que simplemente queremos
leer numeros de un archivo de texto.

```rust
use std::io::{self, BufRead};

/// Read integers from a text file
/// The file should have one number on each line.

fn read_numbers(file: &mut BufRead) -> Result<Vec<i64>, io::Error> {
   let mut numbers = vec![];
   for line_result in file.lines() {
      let line = line_result?;     // reading lines can fail
      numbers.push(line.parse()?); // parsing integers can fail
   }
   Ok(numbers)
}
```

Cuando compilamos el codigo tenemos un error de que no puede convertir un
`std::num::ParseIntError` a un `std::io::Error`, el problema aca es que los dos
fuentes de errores que tenemos son diferentes types. Hay muchas maneras de
solucionarlo, por ejemplo la libreria de imagenes define su propio error
`ImageError` e implementa conversiones desde `io::Error` y otros types de Errores
Tambien podemos utilizar el crate `error-chain`

Una manera mas simple puede ser usar lo que esta hecho ya en la libreria estandar
de Rust, todos los types de errores pueden ser convertidos a el type `Box<std::Error>`
el cual representa "cualquier error". Entonces una manera facil de manejar muchos
fuentes de errores es definir estos types alias:

`type GenError = Box<std::error::Error>;`

`type GenResult<T> = Result<T, GenError>;`

Entonces cambiando el type de retorno de la funcion `read_numbers()` a
`GenResult<Vec<i64>>`. Con este cambio el codigo compila. Y el operador `?`
convierte automaticamente ese type en un `GenError` como necesitamos

## Manejando errores en `main()`

En todos los lugares donde un `Result` es producido, dejar que el error "suba"
hacia el que llama a la funcion es el comportamiento adecuado. Pero si propagamos
lo suficiente eventualmente llegaremos a `main()` y este es donde debemos parar
en `main()` no podemos usar `?` porque su type de retorno no es un `Result` (o
casi siempre)

Comentario:

Se puede hacer que el type de retorno de `main()` sea un `Result` creo que esto
es porque el libro quedo viejo en este tema(recordemos que es del 2015)


## Declarando un type de error personalizado

Supongamos que estamos escribiendo un parser para archivos JSON, y queremos
tener nuestros propios errores. Osea tenemos un struct como esta:

```rust
#derive(Debug, Clone)
pub struct JsonError {
   pub message: String,
   pub line: usize,
   pub column: usize
}
```

Esta `struct` la podemos llamar con `json::JsonError` y cuando queremos tirar
un error de este type podemos escribir:

```rust
return Err(JsonError{
   message: "Expected ']' at the end od an array".to_string(),
   line: current_line,
   column: current_column
})
```

Esto funcionara bien pero si queremos que el error funcione como los errores de
la libreria estandar, entonces tenenmos un poco mas de trabajo que hacer:

```rust
use std;
use std::fmt;

// Errors should be printables
impl fmt::Display for JsonError {
   fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
      write!(f, "{} ({}: {})", self.message, self.line, self.column)
   }
}

// Errors should implement the std::error::Error trait

impl std::error::Error for JsonError {
   fn description(&self) -> &str {
      &self.message
   }
}
```

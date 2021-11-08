# Cap20

## Programacion asincronica

Supongamos que queremos hacer una aplicacion de chat, para ello debemos manejar
las conexiones, la entrada de datos, la salida de datos, los parametros de la red
Manejar todo esto para muchas conexiones puede ser desafiante. Idealmente podemos
comenzar abriendo un thread por cada conexion entrante:

```rust
use std::{net, thread};

let listener = net::TcpListener::bind(address)?;

for socket_result in listener.incomming() {
   let socket = socket_result?;
   let groups = chat_group_table.clone();
   thread::spawn(|| {
      log_error(serve(socket, groups));
   });
}
```
Para cada nueva conexion, esto "spawnmea" un nuevo thread que corre la funcion
`serve()`. Esto funciona bien para un numero pequeño de usuarios pero cuando ese
numero se vuelve un poco mas de 100 empezamos a tener problemas de memoria ya
que cada thread que abrimos consume alrededor de 100KiB de memoria en la stack
Los threads son buenos y necesarios cuando queremos distribuir el trabajo con muchos
procesadores, pero su demanda de memoria son tales que a menudo necesitamos maneras
complementarias que usadas con threads para hacer que el trabajo total por cpu
este balanceado. Pero para este tipo de problemas podemos usar tareas asincronicas
de rust para intercalar muchas actividades independientes sobre un solo thread
o un pool de threads. Las tareas asincronicas son similares a los threads, pero
son mucho mas rapidas para crear, pasar control a si misma mas eficientemente y
tienen un consumo de memoria mucho menor que las de los threads. Generalmente el
codigo asincronico de Rust se ve muy similar al codigo comun, exepto que las operaciones
que pueden ser bloqueantes, como por ejemplo I/O o adquirir Mutexes, necesitan de
ser tratadas de manera diferente. Por ejemplo la version asincronica del codigo
anterior se ve mas o menos asi:

```rust
use async_std::{net, task};

let listener = net::TcpListener::bind(address).await?;

let mut new_connections = listener.incomming();

while let Some(socket_result) = new_connections.next().await {
   let socket = socket_result?;
   let groups = chat_group_table.clone();
   task::spawn(async {
      log_error(serve(socket, groups).await);
   });
}
```

## Desde sincronico a asincronico

Consideremos que pasa cuando llamamos a la siguiente(que no es async completamente
tradicional) funcion

```rust
use std::io::prelude::*;
use std::net;

fn cheapo_request(host: &str, port: u16, path: &str) -> std::io::Result<String> {
   let mut socket = net::TcpStream::conect((host, port))?;
   let request = format!("GET {} HTTP/1.1\r\rHost: {}\r\n\r\n", path, host);
   socket.write_all(request.as_bytes())?;
   socket.shutdown(net::Shutdown::Write)?;

   let mut response = String::new();
   socket.read_to_string(&mut response)?;

   Ok(response)
}
```

Esta funcion abre una conexion TCP a un server web, envia un esqueleto de request de
HTTP en un protocolo que esta en desuso(nos dice que si necesitamos esto deberiamos
utilizar un crate que estan hechos para esto HTTP client como `surf`, `request`)

Lo que pasa con esta funcion es que debe esperar mucho tiempo sin hacer nada hasta
que recibe una respuesta del SO ese tiempo de espera el unico thread que esta
corriendo la aplicacion se bloquea. Como vemos en la firma de la funcion:

```rust
fn cheapo_request(host: &str, port: u16, path: &str) -> std::io::Result<String>
```

esta solo terminara su trabajo cuando recibamos la respuesta en forma de `String`
Si lo que queremos es usar nuestro thread para hacer otras operaciones mientras
el SO hace su trabajo lo que vamos a necesitar es una nueva libreria de I/O que
nos provea una version asincronica de esa funcion


### `Futures`

El aproach de Rust para soportar operaciones asincronicas es introducir un trait,
`std::future::Future`:

```rust
trait Future {
   type Output;
   fn poll(self: Pin<&mut self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

enum Poll<T> {
   Ready(T),
   Pending,
}
```

Un `Future` representa una operacion que podemos testear si ha terminado o no.
el metodo de este `poll` nunca espera por una operacion para que termine: siempre
retorna inmediatamente. Si la operacion esta completa, `poll` retorna `Poll::Ready(ouput)`
donde `output` es el resultado final de la operacion, de otra manera, retorna
`Pending`. Si cuando el `future` poolea de nuevo, el nos promete que nos va a avisar
invocando a una funcion "waker", que es una funcion de `callback` que fue dada
en el `Context`. Llamamos a este modelo como "piñata" de programacion asincronica: la
unica cosa que podemos hacer con un `future` es mirar mediante un `poll` hasta
que haya un valor final

Todos los SO modernos incluyen variantes de sus llamadas de sistemas que pueden
usarse para implementar esto del "poolling". En un entorno Unix y Window$ por ejemplo
si ponemos a un socket de red en un modo no bloqueante, entonces leer y escribir
retornaran un error si ellos pueden ser bloqueados.

Entonces una version asincronica de `read_to_string()` puede tener una firma como
la que sigue:

```rust
fn read_to_string(&mut self, buf: &mut String) -> impl Future<Output=Result<usize>>;
```

Es casi la misma firma que la anterior lo que cambia es el type de retorno:
Esta version asincronica lo que retorna es un `future` de un `Result<usize>`.
Necesitamos poolear este `future` hasta que tengamos un `Ready` de el. Como el
`future` guarda las referencias de `self` y `buf` la firma real que debemos poner
en la funcion `read_to_string` para que funcione es la siguiente:

```rust
fn read_to_string<'a>(&'a mut self, buf: &'a mut String) -> impl Future<Output=Result<usize>> + 'a;
```

Como vemos tenemos que agregar el "lifetime" para indicar que queremos que el `future`
retornado debe "vivir" solo a lo sumo como el valor que `self` y `buf` estan tomando
prestados

El crate `async-std` provee la version asincrona de todas las funciones de la libreria
`std` de I/O que impl los traits `Read`. Este crate como proviene de la libreria
`std` reutiliza mucho de los types de esta y sigue los patrones de diseño de
esta ultima.

Una de las reglas de el trait `Future` es que una vez que el `future` ha retornado
`Poll::Ready` este debe asumir que nunca mas sera "pooleado" de nuevo. Algunos
`futures` solo retornan `Poll::Pending` por siempre si ellos son sobre-pooleados
otros pueden paniquear o se cuelgan(pero no pueden violar todo lo que el lenguaje
se compromete a cumplir de seguridad o que cause UB). El metodo `fuse` del trait
`Future` convierte cualquier `future` a uno que simplemente retorna `Poll::Pending`
para siempre


## Funciones async y la expresion `await`

Ahora la version de `cheapo_request` escrita como una funcion asincronica:

```rust
use async_std::io::prelude::*;
use async_std::net;

async fn cheapo_request(host: &str, port: u16, path: &str) -> std::io::Result<String> {
   let mut socket = net::TcpStream::connect((host, port)).await?;
   let request = format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", path, host);
   socket.write_all(request.as_bytes()).await?;
   socket.shutdown(net::Shutdown::Write)?;

   let mut response = String::new();
   socket.read_to_string(&mut response).await?;

   Ok(response)
}
```

Esto es casi lo mismo que teniamos antes lo unico que cambia es:

 - La funcion comienza con la palabra reservada `async`
 - Usamos el crate `async_std` que es la version asincronica de `TcpStream::connect`
 - Luego de cada llamada que retorna un `future`, el codigo dice `await`. Aunque
   Aunque esto luce como una referencia a el nombre de un campo de una `struct`
   es en realidad una sintaxis especial del lenguaje para esperar hasta que el
   `future` este listo (`Ready`)

En lugar de una funcion ordinaria, cuando llamamos una funcion asincrona esta retorna
inmediatamente, incluso antes de que el body comience la ejecucion de su codigo.
Obviamente el valor de retorno final no se ha completado aun lo que obtenemos es
un `future` de su valor final. Entonces si nosotros ejecutamos el siguiente codigo

```rust
let response = cheapo_request(host, port, path);
```

Entonces `response` sera un `future` de un `std::io::Result<String>` y el
cuerpo de `cheapo_request` no se ha comenzado su ejecucion. No necesitamos
ajustar el type de retorno; Rust automagicamente trata una funcion async `async
fn f() -> T` como una funcion que retorna un future de un `T` y no un `T`
directamente, este type no tiene un nombre lo unico que sabemos es que impl
`Future<Ouput=R>` donde `R` es el type de retorno de la funcion async. En este
sentido, `future`s de funciones async son como los `clusures`: estos tambien
tienen types anonimos generados por el compilador que implementa los traits
`FnOnce`, `FnMut` y `Fn`.

Cuando corremos por primera vez la funcion `cheapo_request` la ejecucion
comienza desde arriba de el cuerpo de la funcion y corre hasta que encuentra el
primer `await` del `future` retornado por `TcpStream::connect()`. La expresion
de `await` pregunta a el `future` de `connect()` y si no esta `Ready` entonces
este retorna un `Poll::Pending` a el que lo llamo, osea que el `future` de la
funcion original `cheapo_request` no estara lista hasta que todos los `await`
internos lo esten y hayan cambiado de estado a `Ready` Una expresion `await`
toma propiedad del el `future` y luego hace el "pooleo", si esta listo entonces
el valor final del `future` es el valor del `await` y la ejecucion continua. De
otra manera retorna el `Poll::Pending` a quien lo haya llamado

Por el momento no se pueden poner metodos `async` en los traits, pero esta planeado
que si se pueda, pero parece que si lo necesitamos hay un crate: `async-trait`
que provee una solucion basada en macros


### LLamando funciones `async` desde codigo sincronico: `block_on`

En cierto sentido las funciones `async` solo "pasan la pelota". Podemos llamar a
la funcion `cheapo_request` desde una funcion comun sincronica(como `main` por ejemplo)
usando la funcion de `async_std` `task::block_on` la cual toma un `future` y "poolea"
hasta que este produce un valor:

```rust
fn main() -> std::io::Result<()> {
   use async_std::task;

   let response = task::block_on(cheapo_request("example.com", 80, "/"))?;
   println!("{}", response);

   Ok(())
}
```

Dado que `block_on` es una funcion sincronica que produce el valor final de una
funcion asincronica, podemos pensarla como que es un adaptador de el mundo
asincronico al mundo sincronico. Pero su naturaleza bloqueante tambien
significa que no debemos usarla nunca dentro de una funcion `async` ya que
bloqueara todo el thread hasta que el valor este listo. Para ello esta `await`

### Spawnmeando Tareas async

La funcion `async_std::block_on` bloquea hasta que obtenemos un valor que esta
`Ready` pero bloquear al thread solo por un `future` no tiene mucho sentido, la
gracia esta en que cuando este en `Pending` sigamos haciendo cosas en el thread
Para esto podemos usar `async_std::task::spawn_local`. Esta funcion toma un `future`
y lo agrega al un "pool". Esta funcion es la analoga asincronica de `std::thread::spawn`

si queremos usar esta funcion debemos poner lo siguiente en el `Cargo.toml`:

```toml
async-std = {version = "1.10.0", features = ["unstable"]}
```

 - `std::thread::spawn(c)`: toma un closure `c` y comienza a correr un thread,
   retornando a `std::thread::JoinHandle` cuyo metodo `join` espera por que termine
   el thread y retorna lo que sea que `c` retorne

 - `async_std::task::spawn_local(f)`: toma el `future` `f` y agrega este a el "pool"
   para que sea "pooleado" cuando el thread actual llame a `block_on()`. `spawn_local`
   retorna su propio type llamado `async_std::task::JoinHandle` que es en si mismo
   un `future`(porque impl el trait) que puede `await` para dar el valor final de `f`

Por ejemplo supongamos que queremos hacer un conjunto de `HTTP` "requests" de
manera concurrente. Este podria ser un intento:

```rust
pub async fn many_request(request: Vec<(String, u16, String)>) -> Vec<std::io::Result<String>> {
   use async_std::task;
   let mut handle = vec![];
   for (host, port, path) in request {
      handle.push(task::spawn_local(cheapo_request(&host, port, &path)));
   }

   let mut results = vec![];
   for handle in handles {
      results.push(handle.await);
   }
   results
}
```

Esta funcion llama a `cheapo_request()` sobre cada elemento del vector `request`
pasando cada llamada de un `future` a `spawn_local`. Este collecta todos los resultados
en el `JoinHandle` en un vector y luego aguarda por cada uno de ellos. Se puede
`await` los `JoinHandle`s en cualquier orden: dado que los `request`s ya han sido
spwameados, sus `futures` seran esperados necesariamente. Por ello todos los `requests`
estan corriendo concurrentemente. Una vez que ellos se hayan completado, `many_request`
retorna el resultado a quien lo ha llamado


Podemos salvar el error que nos tira porque `path` no se puede prestar ya que no
sabe cuanto va a vivir y porque los `futures` tienen un lifetime implicito de `'static`

podemos hacer una version 'owned' de la funcion anterior:

```rust
async fn cheapo_owning_request(host: String, port: u16, path: String) -> std::io::Result<String> {
   cheapo_request(&host, port, &path).await
}
```

Con esos cambios podemos hacer los requests poniendo los hosts url en un vec

```rust
let requests = vec![
   ("example.com".to_string(), 80, "/".to_string())
   ("en.wikipedia.org".to_string(), 80, "/".to_string())
   ("www.red-beam.com".to_string(), 80, "/".to_string())
]
```

una diferencia importante para tener en cuenta entre las tareas asincronicas y
los threads es que cambiar desde un tarea asincronica a otra ocurre solo en las
expresiones `await`s, cuando el `future` que esta siendo esperado retorna un
`Poll::Pending`


### Bloques Async

En adicion a las funciones asincronas, Rust tambien soporta bloques asincronos.
Que es un bloque ordinario de ejecucion que retorna un valor de la ultima expresion
un bloque `async` retorna un `future` de un valor de su ultima expresion. Podemos
usar una expresion `await` dentro de un bloque async. Un bloque async luce como
un bloque ordinario que es precedido por la palabra reservada `async`

```rust
let serve_one = async {
   use async_std::net;
   // listener for connections and accept one
   let listener = net::TcpListener::bind("localhost:8087").await?;
   let (mut socket, _addr) = listener.accept().await?;
   // talk to client on `socket`
};
```

Esto inicializa `serve_one` con un `future` que, cuando es "pooleado" escucha a las
conexiones TCP. En el cuerpo del body no comienza la ejecucion hasta que `serve_one`
se "pollea" solo como una llamada a una funcion async que no comienza su ejecucion
hasta que el `future` es "pooleado"
Los bloques `async` son una manera concisa de separar una seccion del codigo que queremos
que corra de manera asincronica. Por ejemplo en el ejemplo anterior `spawn_local`
requiere que el future tenga un lifetime `'static` por ello definimos una funcion
wrapper que nos da a nosotros un `future` que toma propiedad de sus argumentos
Podemos tener el mismo efecto sin tener que usar esa funcion wrapper simplemente
llamando `cheapo_request` desde un bloque `async`

```rust
pub async fn many_request(requests: Vec<(String, u16, Strin)>) -> Vec<std::io::Result<String>> {
   use async_std::task;
   let mut handles = vec![];
   for (host, port, path) in requests {
      handles.push(task::spawn_local(async move {
         cheapo_request(&host, port, &path).await
      }));
   }
}
```

### Generando funciones async desde bloques async

Bloques asincronicos nos dan otra manera de tener el mismo efecto que con las funciones
asincronicas, con un poco mas de flexibilidad. Por ejemplo, podemos escribir la
funcion `cheapo_request` como una funcion ordinaria que retorna un `Future`


```rust
use std::io;
use std::future::Future;

fn cheapo_reques<'a>(host: &'a str, port: u16, path: &'a str) -> impl Future<Ouput=io::Result<String>> + 'a {
   async move {
      // cuerpo de la funcion anterior...
   }
}
```

Esta segunda version puede ser util cuando queremos hacer algunas operaciones immediatamente
cuando la funcion es llamada, antes de crear el future de su resultado. Por ejemplo:


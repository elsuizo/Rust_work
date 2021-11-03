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
 - Usamos el crate `async_std` que es la version

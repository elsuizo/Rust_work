# Cap20

## Programacion asincronica


Supongamos que queremos hacer una aplicacion de chat, para ello debemos manejar
las conexiones, la entrada de datos, la salida de datos, los parametros de la
red Manejar todo esto para muchas conexiones puede ser desafiante. Idealmente
podemos comenzar abriendo un thread por cada conexion entrante:

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
Los threads son buenos y necesarios cuando queremos distribuir el trabajo con
muchos procesadores, pero su demanda de memoria son tales que a menudo
necesitamos maneras complementarias que usadas con threads para hacer que el
trabajo total por cpu este balanceado. Pero para este tipo de problemas podemos
usar tareas asincronicas de rust para intercalar muchas actividades
independientes sobre un solo thread o un pool de threads. Las tareas
asincronicas son similares a los threads, pero son mucho mas rapidas para crear,
pasar control a si misma mas eficientemente y tienen un consumo de memoria mucho
menor que las de los threads. Generalmente el codigo asincronico de Rust se ve
muy similar al codigo comun, exepto que las operaciones que pueden ser
bloqueantes, como por ejemplo I/O o adquirir Mutexes, necesitan de ser tratadas
de manera diferente. Por ejemplo la version asincronica del codigo anterior se
ve mas o menos asi:

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

Consideremos que pasa cuando llamamos a la siguiente(que no es async
completamente tradicional) funcion

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

Esta funcion abre una conexion TCP a un server web, envia un esqueleto de
request de HTTP en un protocolo que esta en desuso(nos dice que si necesitamos
esto deberiamos utilizar un crate que estan hechos para esto HTTP client como
`surf`, `request`)

Lo que pasa con esta funcion es que debe esperar mucho tiempo sin hacer nada
hasta que recibe una respuesta del SO ese tiempo de espera el unico thread que
esta corriendo la aplicacion se bloquea. Como vemos en la firma de la funcion:

```rust
fn cheapo_request(host: &str, port: u16, path: &str) -> std::io::Result<String>
```
esta solo terminara su trabajo cuando recibamos la respuesta en forma de `String`
Si lo que queremos es usar nuestro thread para hacer otras operaciones mientras
el SO hace su trabajo lo que vamos a necesitar es una nueva libreria de I/O que
nos provea una version asincronica de esa funcion

### `Futures`

El aproach de Rust para soportar operaciones asincronicas es introducir un
trait, `std::future::Future`:

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
el metodo de este `poll` nunca espera por una operacion para que termine:
siempre retorna inmediatamente. Si la operacion esta completa, `poll` retorna
`Poll::Ready(ouput)` donde `output` es el resultado final de la operacion, de
otra manera, retorna `Pending`. Si cuando el `future` poolea de nuevo, el nos
promete que nos va a avisar invocando a una funcion "waker", que es una funcion
de `callback` que fue dada en el `Context`. Llamamos a este modelo como "piñata"
de programacion asincronica: la unica cosa que podemos hacer con un `future` es
mirar mediante un `poll` hasta que haya un valor final

Todos los SO modernos incluyen variantes de sus llamadas de sistemas que pueden
usarse para implementar esto del "poolling". En un entorno Unix y Window$ por
ejemplo si ponemos a un socket de red en un modo no bloqueante, entonces leer y
escribir retornaran un error si ellos pueden ser bloqueados.

Entonces una version asincronica de `read_to_string()` puede tener una firma
como la que sigue:

```rust
fn read_to_string(&mut self, buf: &mut String) -> impl Future<Output=Result<usize>>;
```

Es casi la misma firma que la anterior lo que cambia es el type de retorno: Esta
version asincronica lo que retorna es un `future` de un `Result<usize>`.
Necesitamos poolear este `future` hasta que tengamos un `Ready` de el. Como el
`future` guarda las referencias de `self` y `buf` la firma real que debemos
poner en la funcion `read_to_string` para que funcione es la siguiente:

```rust
fn read_to_string<'a>(&'a mut self, buf: &'a mut String) -> impl Future<Output=Result<usize>> + 'a;
```

Como vemos tenemos que agregar el "lifetime" para indicar que queremos que el
`future` retornado debe "vivir" solo a lo sumo como el valor que `self` y `buf`
estan tomando prestados

El crate `async-std` provee la version asincrona de todas las funciones de la
libreria `std` de I/O que impl los traits `Read`. Este crate como proviene de la
libreria `std` reutiliza mucho de los types de esta y sigue los patrones de
diseño de esta ultima.

Una de las reglas de el trait `Future` es que una vez que el `future` ha
retornado `Poll::Ready` este debe asumir que nunca mas sera "pooleado" de nuevo.
Algunos `futures` solo retornan `Poll::Pending` por siempre si ellos son
sobre-pooleados otros pueden paniquear o se cuelgan(pero no pueden violar todo
lo que el lenguaje se compromete a cumplir de seguridad o que cause UB). El
metodo `fuse` del trait `Future` convierte cualquier `future` a uno que
simplemente retorna `Poll::Pending` para siempre

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
 - Usamos el crate `async_std` que es la version asincronica de
   `TcpStream::connect`
 - Luego de cada llamada que retorna un `future`, el codigo dice `await`. Aunque
   Aunque esto luce como una referencia a el nombre de un campo de una `struct`
   es en realidad una sintaxis especial del lenguaje para esperar hasta que el
   `future` este listo (`Ready`)

En lugar de una funcion ordinaria, cuando llamamos una funcion asincrona esta
retorna inmediatamente, incluso antes de que el body comience la ejecucion de su
codigo. Obviamente el valor de retorno final no se ha completado aun lo que
obtenemos es un `future` de su valor final. Entonces si nosotros ejecutamos el
siguiente codigo

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

Por el momento no se pueden poner metodos `async` en los traits, pero esta
planeado que si se pueda, pero parece que si lo necesitamos hay un crate:
`async-trait` que provee una solucion basada en macros

### LLamando funciones `async` desde codigo sincronico: `block_on`

En cierto sentido las funciones `async` solo "pasan la pelota". Podemos llamar a
la funcion `cheapo_request` desde una funcion comun sincronica(como `main` por
ejemplo) usando la funcion de `async_std` `task::block_on` la cual toma un
`future` y "poolea" hasta que este produce un valor:

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
Para esto podemos usar `async_std::task::spawn_local`. Esta funcion toma un
`future` y lo agrega al un "pool". Esta funcion es la analoga asincronica de
`std::thread::spawn`

si queremos usar esta funcion debemos poner lo siguiente en el `Cargo.toml`:

```toml
async-std = {version = "1.10.0", features = ["unstable"]}
```

 - `std::thread::spawn(c)`: toma un closure `c` y comienza a correr un thread,
   retornando a `std::thread::JoinHandle` cuyo metodo `join` espera por que
   termine el thread y retorna lo que sea que `c` retorne

 - `async_std::task::spawn_local(f)`: toma el `future` `f` y agrega este a el
   "pool" para que sea "pooleado" cuando el thread actual llame a `block_on()`.
   `spawn_local` retorna su propio type llamado `async_std::task::JoinHandle`
   que es en si mismo un `future`(porque impl el trait) que puede `await` para
   dar el valor final de `f`

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

Esta funcion llama a `cheapo_request()` sobre cada elemento del vector
`request` pasando cada llamada de un `future` a `spawn_local`. Este collecta
todos los resultados en el `JoinHandle` en un vector y luego aguarda por cada
uno de ellos. Se puede `await` los `JoinHandle`s en cualquier orden: dado que
los `request`s ya han sido spwameados, sus `futures` seran esperados
necesariamente. Por ello todos los `requests` estan corriendo concurrentemente.
Una vez que ellos se hayan completado, `many_request` retorna el resultado a
quien lo ha llamado

Podemos salvar el error que nos tira porque `path` no se puede prestar ya que
no sabe cuanto va a vivir y porque los `futures` tienen un lifetime implicito
de `'static`

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
Que es un bloque ordinario de ejecucion que retorna un valor de la ultima
expresion un bloque `async` retorna un `future` de un valor de su ultima
expresion. Podemos usar una expresion `await` dentro de un bloque async. Un
bloque async luce como un bloque ordinario que es precedido por la palabra
reservada `async`

```rust
let serve_one = async {
   use async_std::net;
   // listener for connections and accept one
   let listener = net::TcpListener::bind("localhost:8087").await?;
   let (mut socket, _addr) = listener.accept().await?;
   // talk to client on `socket`
};
```

Esto inicializa `serve_one` con un `future` que, cuando es "pooleado" escucha a
las conexiones TCP. En el cuerpo del body no comienza la ejecucion hasta que
`serve_one` se "pollea" solo como una llamada a una funcion async que no
comienza su ejecucion hasta que el `future` es "pooleado" Los bloques `async`
son una manera concisa de separar una seccion del codigo que queremos que corra
de manera asincronica. Por ejemplo en el ejemplo anterior `spawn_local`
requiere que el future tenga un lifetime `'static` por ello definimos una
funcion wrapper que nos da a nosotros un `future` que toma propiedad de sus
argumentos Podemos tener el mismo efecto sin tener que usar esa funcion wrapper
simplemente llamando `cheapo_request` desde un bloque `async`

```rust
pub async fn many_request(requests: Vec<(String, u16, String)>) -> Vec<std::io::Result<String>> {
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

Bloques asincronicos nos dan otra manera de tener el mismo efecto que con las
funciones asincronicas, con un poco mas de flexibilidad. Por ejemplo, podemos
escribir la funcion `cheapo_request` como una funcion ordinaria que retorna un
`Future`


```rust
use std::io;
use std::future::Future;

fn cheapo_reques<'a>(host: &'a str, port: u16, path: &'a str) -> impl Future<Ouput=io::Result<String>> + 'a {
   async move {
      // cuerpo de la funcion anterior...
   }
}
```

Esta segunda version puede ser util cuando queremos hacer algunas operaciones
immediatamente cuando la funcion es llamada, antes de crear el future de su
resultado. Por ejemplo: una nueva version de `cheapo_request` con `spawn_local`
tendria que hacer que la funcion retorne un `future` con lifetime `static` que
capture la propiedad de los argumentos que le son pasados

```rust
fn cheapo_reques(host: &str, port: u16, path: &str) -> impl Future<Output=io::Result<String>> + 'static {
   let host = host.to_string();
   let path = path.to_string();

   asycn move {
      // aca usariamos &*host, port y path
   }
}
```

Como sabemos el `'static` que pusimos en el type de retorno no es necesario ya
que por default todos tienen ese lifetime(cuando tenemos un `impl T`)
Dado que esta version de `cheapo_reques` retorna un `future` que es `'static`,
podemos pasarle directamente a `spawn_local`:

```rust
let join_handle = async_std::task::spawn_local(cheapo_reques("areweasyncyet.rs", 80, "/"));
// otro trabajo para la correr...
let response = join_handle.await?;
```

### Spawnmeando tareas async en un pool de threads

Los ejemplos que vimos hasta ahora pasan casi todo el tiempo esperando por
operaciones de I/O, pero algunas cargas de trabajo son mas un mix de trabajo
del procesador y bloqueo. Cuando nosotros tenemos mucha computacion para hacer
que un solo procesador que no puede mantener el ritmo podemos usar
`async_std::task::spawn` para spawnmear un `future` que esta listo para avanzar

```rust
use async_std::task;

let mut handle = vec![];
for (host, port, path) in request {
   handles.push(task::spawn(async move {
      cheapo_reques(&host, port, &path).await
   }));
}
```

Como `spawn_local` `spawn` retorna un valor `JoinHandle` al que podemos `await`
para obtener el valor del `future` final. Pero a diferencia de `spawn_local`,
el `future` no tiene que esperar a que nosotros llamemos a `block_on` antes de
ser "pooleado". Ni bien uno de los threads desde el "pool" es liberado, este
intentara ser "pooleado". En la practica `spawn` es mas utilizado que
`spawn_local` simplemente por que a la gente le gusta saber su carga de
trabajo, no importa si es un mix de computacion y bloqueo, esta esta balanceada
entre todos los recursos de la maquina

### Pero tu `Future` impl `Send`???

Existe una restriccion que `spawn` impone que `spawn_local` no. Dado que el
`future` es enviado a cualquiera de los threads disponibles, el `future` debe
implementar el trait marker `Send`. Ya presentamos el trait `Send` en el
capitulo "Thread Safety: Send and Sync". Un `future` es `Send` solo si los
valores que contiene son `Send`: todos los argumentos de funciones, variables
locales y hasta valores temporarios anonimos deben ser seguros de mover a otro
thread

Podemos chocarnos facilmente con estas restricciones por accidente. Por ejemplo,
el siguiente codigo luce inocente:

```rust
use async_std::task;
use std::rc::Rc;

async fn reluctant() -> String {
   let string = Rc::new("ref counting string".to_string());
   some_asychronous_thing().await;
   format!("You splendid string: {}", string);
}

task::spawn(reluctant());
```

Como buena funcion asincronica necesita manejar un `future` type para guardar la
informacion para que la funcion pueda seguir desde la expresion `await`,
entonces el `future` podria al menos algunas veces contener un valor
`Rc<String>`. Dado que los punteros `Rc` no pueden compartirse entre threads
entonces el `future` no puede impl `Send`

Hay dos maneras de resolver el problema:

Una es haciendo una restriccion al scope del valor que no es `Send` y asi hacer
que no forme parte del `future`

```rust
async fn reluctant() -> String {
   let return_value = {
      let string = Rc::new("ref-counting string".to_string());
      format!("Your splendid string: {}", string);
      // luego de aca el string no esta mas desaparece porque se llama a Drop
   };

   some_asychronous_thing().await;
   return_value
}
```

Otra solucion es simplemente usar un punteros que si impl `Send` como `Arc`

Aunque eventualmente vamos a aprender a reconocer y evitar types que no son
`Send` estos pueden ser un poco sorpresivos a primera vista. Por ejemplo en
codigo viejo de Rust algunas veces se puede ver el uso de `Result` types
genericos como esto:

```rust
// No recomendable es estooo
type GenericError = Box<dyn std::error::Error>;
type GenericResult<T> = Result<T, GenericError>;
```

Este error generico usa un trait object para guardar un valor de cualquier type
que implementa `std::error::Error`, pero no pone ninguna restriccion futura
sobre el: si alguien tiene un valor que es no-`Send` que impl `Error` podria
convertirlo a un valor boxeado de ese type a un `GenericError`. Por estas
posibilidades, entonces es que `GenericError` es no-`Send` y el siguiente codigo
no funcionara:

```rust
fn some_fallible_thing() -> GenericResult<i32> {
   // ...
}

// esta funcion su `future` no puede impl `Send`
async fn unfortunate() {
   match some_fallible_thing() {
      Err(error) => {
         report_error(error);
      },
      Ok(output) => {
         // ... esta vivo a traves de este await
         use_output(output).await;
      }
   }
}

// y asi este `spawn` es un error
async_std::task::spawn(unfortunate());
```

### Codigo que tarda tiempo en ejecutarse: `yield_now` y `spawn_blocking`

Para un `future` para compartir su thread con otras tareas, su metodo `poll`
debe siempre retornar tan pronto como pueda. Pero si estamos trabajando con un
codigo que tienen mucha carga computacional este puede llegar a tardar un tiempo
largo hasta alcanzar el proximo `await`, haciendo que las las otras tareas
asincronicas esperen mas de lo que querriamos para su uso en un thread. Una
manera de evitar esto es simplemente `await` a algo ocacional. La funcion
`async_std::task::yield_now` retorna un `future` simple designado para esto:

```rust
while computation_not_done() {
   do one medium-sized step of computation...
   async_std::task::yield_now().await;
}
```

La primera vez que el `future` de `yield_now` es "pooleado" este retorna
`Pool::Pending` pero dice que merece la pena "poolear" de nuevo pronto. El
efecto es que la llamada asincronica deja el thread y asi otras tareas tienen la
chance de correr, pero la llamada que acabamos de hacer va a tener una nueva
oportunidad pronto. La segunda vez que es llamada `yield_now` este retorna un
`Pool::Ready(())` y la funcion `async` puede resumir la ejecucion

Pero esta aproximacion no siempre es realizable, sin embargo. Si estamos usando
un crate externo para hacer la computacion costosa y llamando a codigo C/C++
podria ser no conveniente de cambiar el codigo a mas amistoso para operaciones
async. Porque es dificil de asegurar que cada camino del algoritmo pase por un
`await` de tiempo en tiempo.

Para casos como estos, podemos usar `async_std::task::spawn_blocking`. Esta
funcion toma un closure, comienza corriendo sobre su propio thread y retorna un
`future` de su valor de retorno. Codigo asincronico pueden `await` ese `future`
cediendo su thread a otras tareas hasta que la computacion costosa se haya
realizado. Poniendo el trabajo duro en un solo thread podemos hacer que el
sistema operativo tome los recaudos para que compartirlo su procesamiento sea
mas facil

Por ejemplo supongamos que necesitamos chequear passwords dados por usuarios
contra versiones hashed que tenemos guardadas en una base de datos. Para
seguridad, verificar los passwords necesitan un costo computacional alto, ya que
si aunque un hacker tenga acceso a esa base de datos, este le seria casi
imposible probar trillones de posibles passwords para ver si alguno matchea. El
crate `argonautica` provee una funcion de hash designada especificamente para
esto: un hash de este crate toma bastante tiempo en verificarla. Usando
`argonautica` (version 0.2) en nuestra aplicacion asincronica


```rust
async fn verify_password(password: &str, hash: &str, key: &str) -> Result<bool, argonautica::Error> {
   // hacemos copias de los argumentos, entonces el closure puede ser 'static
   let password = password.to_string();
   let hash = hash.to_string();
   let key = key.to_string();

   async_std::task::spawn_blocking(move || {
      argonautica::Verifier::default()
      .with_hash(hash)
      .with_password(password)
      .with_secret(key)
      .verify()
   }).await
}
```

Entonces esto retorna un `Ok(true)` si el password matchea el hash, dada una
key. Haciendo la verificacion en el closure pasado a `spawn_blocking` ponemos la
computacion costosa sobre su propio thread asegurandonos que esto no afecte la
experiencia de los otros usuarios

### Comparando los disenios asincronicos de otros lenguajes

En muchos aspectos el approach de Rust a el problema asincronico es parecido a
como lo resolvieron otros lenguajes, por ejemplo javascript, C# y Rust tienen la
expresion await para sus funciones y todos estos lenguajes tienen un valor que
representa una computacion que esta incompleta: Rust los llama `futures`
javascript `promises` y C# los llama `tasks` pero todos ellos representan un
valor que puede que tengamos que esperar para conseguirlo.

La manera de hacer "pooling" si es distinto con respecto a otros lenguajes en
javascript o C# las funciones comienzan a correr ni bien son llamadas y existe
un loop de eventos global que resume a las funciones async suspendidas cuando
los valores que esta esperando se vuelven disponibles. En Rust en cambio las
llamadas async no hacen nada hasta que nosotros utilizamos funciones como
`block_on`, `spawn` o `spawn_local` que van a ocuparse de "poolear" y dirigir el
trabajo que en otros lenguajes se encarga el "event loop"

Ya que Rust hace que el progamador elija un `executor` para hacer el "pooleo" de
los `futures` Rust no necesita de un "event loop" global construido en el
lenguaje El crate `async_std` ofrece las funciones de `executor` que hemos
estado utilizando en este capitulo, pero otro crate como `tokio` ofrece otra
gama de `executors`. Tambien en el final de este capitulo vamos a implementar
nuestro propio `executor` Podemos utilizar los tres variantes en un mismo
programa

### Un cliente real asinconico HTTP
Aqui vamos a reescribir a la nuestra funcion `many_request` haciendo uso de uno
de los tantos crates que existen para traer informacion de la web.

```rust
/// cliente HTTP basado en el codigo del libro "Programming Rust"

pub async fn many_requests(urls: &[String]) -> Vec<Result<String, surf::Error>> {
    let client = surf::Client::new();
    let mut handles = vec![];

    for url in urls {
        let request = client.get(&url).recv_string();
        handles.push(async_std::task::spawn(request));
    }

    let mut results = vec![];
    for handle in handles {
        results.push(handle.await);
    }
    results
}

fn main() {
    let requests = &[
        "http://example.com".to_string(),
        "https://www.red-bean.com".to_string(),
        "https://en.wikipedia.org/wiki/Main_Page".to_string(),
    ];
    let results = async_std::task::block_on(many_requests(requests));

    for result in results {
        match result {
            Ok(response) => println!("***{:}\n", response),
            Err(err) => eprintln!("error: {}\n", err),
        }
    }
}
```

Usando un solo cliente `surf::Client` para hacer todos nuestras requests nos
deja reusar las conexiones HTTP si muchas de ellas estan dirigidas al mismo
server. Y no necesitamos `async_block` porque `recv_string` es un metodo
asincrono que retorna un `future` que implementa `Send + 'static` por ello
podemos pasarle este `future` directamente a la funcion `spawn`


### Un client y server asincronico

Es tiempo de tomar las ideas principales que hemos discutido hasta aqui y
juntarlas en un programa que funcione, en gran medida las aplicaciones
asincronas no hacen acordar a aplicaciones ordinarias que manejan muchos
threads, pero ahora hay nuevas oportunidades para hacer que el codigo sea mas
legible compacto y expresivo

En esta seccion vamos a hacer un server y un client. En la vida real este tipo
de programas son realmente complicados mas que nada la parte de seguridad. Nos
vamos a enfocar solo en la parte de seguridad que va desde como manejamos las
reconecciones a la privacidad y la moderacion del chat.

En particular lo que queremos es manejar bien lo que se conoce como
`backpressure` Por esto lo que queremos decir es que si un client tiene una
conexion lenta o tira su conexion por completo esto no puede afectar a los
otros clientes para nada a la hora de intercambiar mensajes a su propio ritmo.
Y dado que clientes que tienen conexiones lentas no deberian hacer que el
server gaste memoria de mas reteniendo sobre ella el backup de mensajes,
nuestro server no se va a ocupar de ello, ya que los va a eliminar para los
clientes que no puedan seguir con la conexion, pero si los vamos a notificar
que su conexion se ha caido(un chat real deberia logear los mensajes en un
disco fisico para que los clientes puedan volver a tener lo que escribieron)

El codigo lo pongo en una carpeta aparte llamada `async-chat-book`

Como vemos dependemos de cuatro crates:

 - el `async_std`: que como vimos es una coleccion de primitivos para hacer I/O
   de manera asincronica

 - El crate `tokio` que es otra coleccion de primitivas asincronas como
   `async_std` una de las mas maduras. Es muy utilizada pero requiere un poco mas
   de cuidado a la hora de usarla que `async_std`. Es un crate grande pero podemos
   desde el `Cargo.toml` especificar que solo vamos a usar cierto sub-system de el
   Cuando recien comenzaba esto de async en Rust la gente trataba de evitar a las
   dos crates en un mismo programa, pero los dos proyectos han cooperado para que
   se pueda hacer sin problemas
 - Los crates `serde` y `serde_json`: Que como sabemos son convenientes para parsear
   archivos json


El proyecto usa el viejo truco de usar la carpeta `src/bin` ademas de tener la
libreria principal que como siempre se pone en `src/lib.rs` con su submodulo
`src/utils.rs` que tambien incluye dos ejecutables:

La estructura del proyecto es la siguiente:

```text
src
├── bin
│   ├── client.rs
│   └── server
│       ├── connection.rs
│       ├── group.rs
│       ├── group_table.rs
│       └── main.rs
├── lib.rs
└── utils.rs
```

 - `src/bin/client.rs`: es un archivo solo ejecutable para el cliente de chat
 - `src/bin/server`: es el ejecutable del server que se compone de cuatro archivos
   `main.rs` contiene la funcion principal `main` y tenemos tres submodulos:
   `conection.rs`, `group.rs` y `group_table.rs`

Luego para correr los binarios que usan a la libreria que esta en `lib.rs`
simplemente hacemos:

```bash
cargo run --release --bin server --localhost:8088
cargo run --release --bin client --localhost:8088
```

Donde como vemos la bandera `--bin` le indica cual binario tiene que correr

#### Los types de `Error` y `Result`

El modulo del crate de la libreria en el archivo `src/utils.rs` define los
types de `Result` y `Error` que vamos a usar en toda la aplicacion

```rust
use std::error::Erorr;

pub type ChatError = Box<dyn Error + Send + Sync + 'static>;
pub type ChatResult<T> = Result<T, ChatError>;
```

Como dijimos anteriormente necesitamos que los errores sean lo mas generales
posibles para que despues no tengamos problemas que no impl los metodos que
hacen que podamos pasarlos entre threads. Los crates `async_std`, `serde_json`
y `tokio` definen sus propios types de errores, pero el operador `?` puede
automagicamente convertirlos a un `ChatError` usando la implementacion de la
libreria estandar del trait `From` que puede convertir cualquier type de error.
En una aplicacion real nos recomiendan que usemos el crate `anyhow` el cual
provee types para errores y `Result` similares a los que definimos pero ademas
nos ofrece mas posibilidades mas alla de lo que hicimos nosotros


#### El protocolo

La libreria captura nuestro protocolo de chat en estos dos types, definidos en
`lib.rs`

```rust
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod utils;

// TODO(elsuizo:2021-11-12): no podemos reemplazar a los types Post y Message por un type solo que
// sea mas generico y que tenga un builder???

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum FromClient {
    Join {
        group_name: Arc<String>,
    },
    Post {
        group_name: Arc<String>,
        message: Arc<String>,
    },
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum FromServer {
    Message {
        group_name: Arc<String>,
        message: Arc<String>,
    },
    Error(String),
}
//-------------------------------------------------------------------------
//                        testing
//-------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    // testeamos que la serializacion funciona bien en ambos sentidos
    #[test]
    fn test_from_client_json() {
        use std::sync::Arc;

        let from_client = FromClient::Post {
            group_name: Arc::new("Dogs".to_string()),
            message: Arc::new("Samoyeds rock!!!".to_string()),
        };
        let json = serde_json::to_string(&from_client).unwrap();
        assert_eq!(
            json,
            r#"{"Post":{"group_name":"Dogs","message":"Samoyeds rock!!!"}}"#
        );

        assert_eq!(
            serde_json::from_str::<FromClient>(&json).unwrap(),
            from_client
        );
    }
}
```

El `enum` `FromClient` representa el paquete que un client puede enviar al
server puede perdir unirse a una sala y postear mensajes a cualquier sala que
se ha unido

`FromServer` representa lo que el server puede enviar de vuelta: los mensajes
posteados a cierto grupo y los mensajes de errores. Usando un "reference
counted" `Arc<String>` en lugar de un `String` comun nos ayuda a que el server
evite hacer copias costosas de los strings mientras se manejan los grupos y se
distribuyen mensajes

#### Tomando la entrada del usuario: Streams asincronicos

Nuestro cliente de chat tiene como principal responsabilidad leer los comandos
que pone el usuario y enviar los correspondientes paquetes a el server. Manejar
una interface de usuario adecuada no es el proposito de este ejemplo; por ello
vamos a hacer lo mas simple posible para que las cosas funcionen: leer lineas
directamente desde la entrada estandar. El siguiente codigo va en el archivo
`src/bin/client.rs`

```rust
async fn send_commands(mut to_server: net::TcpStream) -> ChatResult<()> {
    println!(
        "Commands: \n\
             join GROUP\n\
             post GROUP MESSAGE...\n\
             Type Control-D(on UNIX) or Control-Z(on Windows)\
             to close connection"
    );

    let mut command_lines = io::BufReader::new(io::stdin()).lines();
    while let Some(command_result) = command_lines.next().await {
        let command = command_result?;
        let request = match parse_command(&command) {
            Some(request) => request,
            None => continue,
        };
        utils::send_as_json(&mut to_server, &request).await?;
        to_server.flush().await?;
    }
    Ok(())
}
```

Esta funcion llama a `async_std::io::stdin` para obtener un handle asincronico
sobre la entrada estandar, lo envolvemos en un `async_std::io::BufReader` para
asi "bufferearlo" y entonces llamamos a `lines` para procesar la entrada del
usuario linea a linea. Trata de parsear cada linea como un commando
correspondiente a la `struct` `FromClient` y si es correcto envia el valor al
server, si el usuario envia un commando que no es reconocido, `parse_command`
imprime un mensaje de error y retorna None, entonces `send_commands` puede
volver a correr el loop de nuevo Y si el usuario ingresa un final de
archivo(presionando C-d) entonces la lineas de stream retornan `None` y
`send_commands` retorna

El metodo asincronico del type `BufReader` es interesante. Este no puede
retornar un iterador, la manera que la libreria estandar lo hace es: Como
sabemos para el type `Iterator` el metodo `next` no es asincronico, entonces
llamando `commands.next()`

podria bloquear el thread hasta que la proxima linea este lista. En cambio,
`lines` retorna un `stream` de valores `Result<String>`. Un Stream es el
analogo asincronico de un iterador: este produce una secuencia de valores sobre
demanda en una manera asincronica amigable, en la definicion del trait una de
las funciones importantes es `poll_next`, los Streams tienen asociado un type
`Item` y usan `Option` para indicar cuando una secuencia ha terminado, pero
como un `future` puede ser "pooleado" para obtener el proximo item osea que
podremos llamar a `poll_next` hasta que esta retorne `Poll::Ready` El metodo
`poll_next` es feo de utilizar directamente, pero generalmente no necesitamos
hacerlo ya que como `Iterators` los streams tienen una amplia coleccion de
metodos como `filter`, `map` ...etc Poniendo todas estas piezas juntas
`send_commands` consume el stream de input de lineas haciendo un loop sobre los
valores producidos por el stream usando `next` con un `while let`. Cuando
trabajamos con Streams es importante recordar importar el prelude de
`async_std`


#### Enviando paquetes

Para transmitir los paquetes sobre una red de socket(no se si esta bien esta
traduccion...) nuestro client y server usan la funcion `send_as_json` desde
nuestra crate `utils`

```rust
pub async fn send_as_json<S, P>(outbound: &mut S, packet: &P) -> ChatResult<()>
where
    S: async_std::io::Write + Unpin,
    P: Serialize,
{
    let mut json = serde_json::to_string(&packet)?;
    json.push('\n');
    outbound.write_all(json.as_bytes()).await?;
    Ok(())
}
```

Como vemos esta funcion es bastante flexible ya que el type de paquete a ser
enviado puede ser cualquier type `P` que impl `Serialize`. La restriccion de
`Unpin` sobre `S` es requerido para usar el metodo `write_all`


#### Recibiendo packets: Mas Streams asincronicos

Para recibir paquetes nuestro server y cliente necesitan correr la siguiente
funcion desde el modulo `utils` para recibir valores desde `FromClient` y
`FromServer` desde un buffer asincronico TCP osea un
`async_std::io::BufReader<TcpStream>`

```rust
use serde::de::DeserializeOwned;

pub fn receive_as_json<S, P>(inbound: S) -> impl Stream<Item = ChatResult<P>>
where
    S: async_std::io::BufRead + Unpin,
    P: DeserializeOwned,
{
    inbound.lines().map(|line_result| -> ChatResult<P> {
        let line = line_result?;
        let parsed = serde_json::from_str::<P>(&line)?;
        Ok(parsed)
    })
}
```

Como `send_as_json` es una funcion generica sobre los types de la entrada y el
`packet`

a- El type del stream `S` debe implementar `async_std::io::BufRead`, el analogo
   a `std::io::BufRead` que representa un input de bytes streams

 - El type del `packet` `P` debe implementar `DeserializeOwned` que es una variante
   del trait de `serde` `Deserialize`. Por eficiencia `Deserialize` puede producir
   valores `&str` y `&[u8]` que prestan su contenido directamente desde el buffer
   desde donde fueron deserializados, para evitar copiar datos. En nuestro caso
   eso no es bueno porque no necesitamos devolver los valores deserializados a
   quien ha llamado entonces debe poder vivir al menos como los buffers a los que
   estamos parseando. Un type que impl `DeserializeOwned` es siempre independiente
   del buffer del cual se esta deserializando

Llamando a `inbound.lines()` nos da un `Stream` de valores
`std::io::Result<String>` Cuando usamos el adaptador `map` para aplicar un
closure a cada item, el manejo de errores y parseo de cada linea como un
formato `json` de un valor de type `P` Esto nos da un stream de valores
`ChatResult<P>`, los cuales retornamos directamente

La el type de retorno de la funcion es:

```rust
impl Stream<Item = ChatResult<P>>
```

Esto indica que vamos a retornar algun type que produce una secuencia de
valores asincronicos `ChatResult<P>`, pero quien llama a la funcion no puede
decirnos exactamente cual es el type. Dado que el closure que le pasamos a el
`map` tiene un type anonimo de todas maneras osea que este es el type mas
especifico que puede retornar

Notemos que `receive_as_json` no es una funcion asincronica, es una funcion
ordinaria que retorna un valor `async` un `Stream`. Entender bien como funciona
la mecanica de las funciones asincronicas en Rust es mas de poner `async`s y
`await`s por todos lados hasta que compile ya que habre el potencial para
definiciones mas flexibles como la anterior que toman todas las ventajas del
lenguaje

```rust
use async_chat_book::FromServer;

async fn handle_replies(from_server: net::TcpStream) -> ChatResult<()> {
    // aca leemos lo que nos trajo la conexion
    let buffered = io::BufReader::new(from_server);
    // aca lo convertimos a json
    let mut reply_stream = utils::receive_as_json(buffered);
    // aca es cuando usamos la magia de los Streams(que son como iterators pero asincronicos)
    // capaz que en proximas versiones de Rust podamos hacer un simple for aca...
    while let Some(reply) = reply_stream.next().await {
        match reply? {
            FromServer::Message {
                group_name,
                message,
            } => {
                println!("message posted to: {}: {}", group_name, message);
            }
            FromServer::Error(message) => {
                println!("error from server: {}", message)
            }
        }
    }
    Ok(())
}
```

Esta funcion toma un socket que recibe datos desde el server lo "wrappea" en un
`BufReader` (notemos que es la version `async_std`) y luego lo pasa a
`receive_as_json` para obtener un stream de valores que vienen de `FromServer`

#### La funcion principal del `client`

Dado que hemos presentado ambas funciones `send_commands` y `handle_replies`
podemos mostrar la funcion principal del client, que esta siempre en
`src/bin/client.rs`

```rust
use async_std::task;

fn main() -> ChatResult<()> {
    let address = std::env::args().nth(1).expect("Usage: client ADDRESS:PORT");

    task::block_on(async {
        let socket = net::TcpStream::connect(address).await?;
        socket.set_nodelay(true)?;

        let to_server = send_commands(socket.clone());
        let from_server = handle_replies(socket);
        from_server.race(to_server).await?;

        Ok(())
    })
}
```

Habiendo obtenido el la direccion del server desde la linea de comandos, `main`
tiene una serie de funciones asincronicas que tendria que llamar entonces lo que
hacemos es envolver ese codigo en un bloque `async` que es pasado a la funcion
`block_on` Una vez que la conexion se establece, lo que queremos es que
`send_commands` y `handle_replies` corran en tandem, para que podamos ver los
mensajes de los otros cuando estemos tipeando. Si entramos el "end-of-file" o si
la conexion al server se cae, el programa debe salirse.

Dado que lo que hemos hecho siempre en este capitulo es del estilo:

```rust
let to_server = task::spawn(send_commands(socket.clone()));
let from_server = task::spawn(handle_replies(socket));

to_server.await?;
from_server.await?;
```

<!-- NOTE(elsuizo: 2023-05-22): aca usamos el metodo race que es importante -->
Pero como nosotros hacemos `await` a ambos de los `join handles` eso nos da a
nosotros un programa que finaliza una vez que ambas tareas hallan finalizado. Lo
que queremos en realidad es que finalice ni bien una de las dos tareas ha
finalizado. Por ello usamos el metodo `race` en la linea:
`from_server.race(to_server)` que retorna un nuevo `future` que "pollea" los dos
`from_server` y `to_server` y retorna un `Poll::Ready(v)` ni bien alguna de las
dos haya finalizados o se convierta en `Ready` los dos `futures` deben tener el
mismo type de retorno. El `future` que no se completa se descarta

Este metodo junto con muchos otros son definidos en el trait
`async_std::prelude::FutureExt` el cual cuando importamos el `prelude` se nos
hace visible para usarlos


#### La funcion `main` del server

```rust
use async_chat_book::utils::ChatResult;
use async_std::prelude::*;
use std::sync::Arc;

mod connection;
mod group;
mod group_table;

use connection::serve;

fn main() -> ChatResult<()> {
    let address = std::env::args().nth(1).expect("Usage: server ADDRESS");
    let chat_group_table = Arc::new(group_table::GroupTable::new());

    async_std::task::block_on(async {
        // este codigo es el mismo que vimos en la introduccion del capitulo
        use async_std::{net, task};
        let listener = net::TcpListener::bind(address).await?;

        let mut new_connections = listener.incoming();
        while let Some(socket_result) = new_connections.next().await {
            let socket = socket_result?;
            let groups = chat_group_table.clone();
            task::spawn(async {
                log_error(serve(socket, groups).await);
            });
        }
        Ok(())
    })
}

fn log_error(result: ChatResult<()>) {
    if let Err(err) = result {
        eprintln!("Error: {}", err)
    }
}
```

Esta funcion nos recuerda un poco a la funcion `main` del cliente: porque hace
un poco de setup y luego llama a un `block_on` para correr un bloque `async` que
es el que hace el trabajo. Para manejar las conexiones que vienen del cliente,
crea un socket de `TcpListener` cuyo metodo entrante retorna un stream de
valores `std::io::Result<TcpStream>`

Por cada conexion que se produce, spawnmeamos una tarea async corriendo la
funcion `connction::serve`. Cada tarea tambien recibe una referencia a un valor
`GroupTable` que representa la lista de grupos de chats del server, compartidas
por todas las conexiones via un `Arc` (que recordemos que es la version `Send`
de `Rc` "reference counting")

Si la conexion retorna un error lo que se hace es un log(en lugar de paniquear)
y dejamos que la tarea salga. Las otras conexiones continuaran corriendo como
siempre

### Manejando conexiones en el chat: Mutexes `async`

Aqui vemos el caballo de batalla del server, la funcion `serve` del modulo
`connection` en `/src/bin/server/connection.rs`

```rust
/// Handle a single client's connection.

use async_chat::{FromClient, FromServer};
use async_chat::utils::{self, ChatResult};
use async_std::prelude::*;
use async_std::io::BufReader;
use async_std::net::TcpStream;
use async_std::sync::Arc;

use crate::group_table::GroupTable;

pub async fn serve(socket: TcpStream, groups: Arc<GroupTable>)
                   -> ChatResult<()>
{
    let outbound = Arc::new(Outbound::new(socket.clone()));

    let buffered = BufReader::new(socket);
    let mut from_client = utils::receive_as_json(buffered);
    while let Some(request_result) = from_client.next().await {
        let request = request_result?;

        let result = match request {
            FromClient::Join { group_name } => {
                let group = groups.get_or_create(group_name);
                group.join(outbound.clone());
                Ok(())
            }

            FromClient::Post { group_name, message } => {
                match groups.get(&group_name) {
                    Some(group) => {
                        group.post(message);
                        Ok(())
                    }
                    None => {
                        Err(format!("Group '{}' does not exist", group_name))
                    }
                }
            }
        };

        if let Err(message) = result {
            let report = FromServer::Error(message);
            outbound.send(report).await?;
        }
    }

    Ok(())
}
```

Esta es casi un espejo de lo que hace la funcion del `client` `handle_replies`,
el codigo mas importante es el loop que maneja el stream que viene desde
`FromClient` que es para hacer un buffer de stream `TCP` con la funcion
`receive_as_json`. Si ocurre un error vamos a generar un `FromServer::Error`
para transmitir las malas noticias al client

Ademas de los mensajes de errores, el client deberia recibir los mensajes desde
el grupo de chat en los cuales se ha unido, entonces la conexion al client
necesita ser compartida con cada grupo. Podemos simplemente darle a cada uno un
clone de el `TcpStream`, pero si dos de esas fuentes intentan escribir un packet
al socket al mismo tiempo, su salida puede ser que la veamos intercalada y el
cliente podria terminar recibiendo un json distorcionado. Por ello necesitamos
hacer que las conexiones sean seguras en el sentido concurrente

Esto es manejado por el type `Outbound` definido en el archivo:
`src/bin/server/connection.rs` de la siguiente manera:

```rust
use async_std::sync::Mutex;

pub struct Outbound(Mutex<TcpStream>);

impl Outbound {
    pub fn new(to_client: TcpStream) -> Self {
        Self(Mutex::new(to_client))
    }

    pub async fn send(&self, packet: FromServer) -> ChatResult<()> {
        let mut guard = self.0.lock().await;
        utils::send_as_json(&mut *guard, &packet).await?;
        guard.flush().await?;
        Ok(())
    }
}
```

Cuando es creado un valor `Outbound` toma el ownership de un `TcpStream` y lo
wrapea en un `Mutex` para asegurar que solo una tarea pueda usarlo en ese
momento. Despues vemos que la funcion `serve` wrappea cada `Outbound` en un
`Arc` entonces que todos los grupos usen la misma instancia de `Outbound`
compartida. Una llamada a `Outbound::send` primero hace un `lock` al mutex
retornando un valor de guarda que es desreferenciado en la funcion `send_as_json`
y finalmente hacemos un `flush` para asegurar que no languidesera a medio
transmitir en algun buffer


### El `Group Table`: Mutexes sincronicos

En el codigo anterior utilizamos el `Mutex` asincrono pero no siempre tenemos
que hacerlo. A menudo no hay necesidad de esperar nada mientras tenemos un mutex
y el lock no es practicamente bloqueante. En esos casos el `Mutex` de la
libreria estandar puede ser mucho mas eficiente. Nuestro type del chat server
`GroupTable` nos muestra esto:

```rust
use crate::group::Group;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct GroupTable(Mutex<HashMap<Arc<String>, Arc<Group>>>);

impl GroupTable {
    pub fn new() -> GroupTable {
        GroupTable(Mutex::new(HashMap::new()))
    }

    pub fn get(&self, name: &String) -> Option<Arc<Group>> {
        self.0.lock()
            .unwrap()
            .get(name)
            .cloned()
    }

    pub fn get_or_create(&self, name: Arc<String>) -> Arc<Group> {
        self.0.lock()
            .unwrap()
            .entry(name.clone())
            .or_insert_with(|| Arc::new(Group::new(name)))
            .clone()
    }
}
```

Como vemos un `GroupTable` es simplemente una `HashMap` que esta protegida por
un `Mutex`, mapeando nombres de grupos de chat con grupos, ambos manejados
usando "reference counted pointers". Los metodos `get` y `get_or_create`
"lockean" el mutex y hacen algunas pocas operaciones de hashtables. Como vemos
aqui utilizamos el mutex comun de la libreria estandar ya que el mutex es solo
locked para hacer unas operaciones que son simples no justifica utilizar toda la
maquinaria async

Ahora si nuestro chat server lo utilizan millones de usuarios y el mutex
de `GroupTable` se convierte en el cuello de botella del proyecto entonces si
deberiamos pensar una alternativa como por ejemplo lo que nos ofrece el crate
"dashmap"


### Grupos de Chat: los channels de tokio

En nuestro server, el type `group::Group` representa un grupo de chat, este type
solo tiene que soportar los dos metodos que `connections::serve` llama: `join`
para agregar un nuevo miembro y `post` para postear un mensaje. Cada mensaje
posteado necesita de ser distribuido a todos los miembros. Aca es donde
podriamos tener el problema de "backpressure" que mencionamos, donde hay muchas
necesidades en tension unas con otras:

 - Si un miembro no puede seguir el ritmo con los mensajes siendo posteados al
   grupo(por ejemplo si tenemos una conexion lenta) otros miembros en el grupo
   no deberian verse afectados

 - Incluso si un miembro se queda atras, deberia poder reconectarse a la
   conversacion y continuar participando de alguna manera

 - La memoria que gastamos guardando mensajes no deberia crecer sin limite

Ya que estas son los challenges mas comunes que nos encontramos cuando
implementamos comunicaciones muchos-a-muchos, el crate tokio provee un type para
hacer brocast que implementa un conjunto razonable de trade-off. Un channel
brocast es una queue de valores (en nuestro caso, mensajes de chat) que nos
permiten cualquier numero diferente de threads o task para enviar y recibir
valores. Se llama canal "bradcast" porque todos los consumidores obtienen su
propia copia de cada valor enviado(por ello el valor enviado debe implementar
`Clone`)

Normalmente un broadcast channel retiene el mensaje en la queue hasta que todos
los consumidores han recibido su copia, pero si el largo de la queue podria
exceder la capacidad maxima del canal, especificada cuando es creada el mensaje
mas viejo es sacado de la queue. Por ejemplo tenemos un channel que tiene una
capacidad maxima de 16 valores. Hay dos senders que van poniendo mensajes en la
queue y cuatro receptores que los van sacando(o mas precisamente copiandolos) el
receptor B tiene 14 mensajes por recibir, el receptor C tiene 7 y el D ya tiene
a todos los mensajes, en cambio el A se ha quedado atras y 11 mensajes que no
vio se han tirado de la queue, entonces la proxima vez que quiera recibir un
mensaje fallara retornando un error indicando la situacion y se correra a el
final actual de la queue

Nuestro server de chat representa cada grupo de chat como un canal de broadcast
llevando valores `Arc<String>` posteando mensajes a todos los miembros del grupo

Aqui esta la definicion del type `group::Group` en `src/bin/server/group.rs`

```rust
use async_std::task;
use crate::connection::Outbound;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct Group {
    name: Arc<String>,
    sender: broadcast::Sender<Arc<String>>
}

impl Group {
    pub fn new(name: Arc<String>) -> Group {
        let (sender, _receiver) = broadcast::channel(1000);
        Group { name, sender }
    }

    pub fn join(&self, outbound: Arc<Outbound>) {
        let receiver = self.sender.subscribe();

        task::spawn(handle_subscriber(self.name.clone(),
                                      receiver,
                                      outbound));
    }

    pub fn post(&self, message: Arc<String>) {
        // This only returns an error when there are no subscribers. A
        // connection's outgoing side can exit, dropping its subscription,
        // slightly before its incoming side, which may end up trying to send a
        // message to an empty group.
        let _ignored = self.sender.send(message);
    }
}
```

Como vemos un type `Group` guarda el nombre de el grupo de chat junto con un
`broadcast::Sender` que representa quien es que manda el mensaje al grupo de
chat. Cuando creamos un nuevo type llamamos a la funcion `bradcast::channel`
para crear el canal y como parametro le pasamos cual es la capacidad maxima del
mismo, que en este caso son 1000 mensajes

Para agregar nuevos miembros al grupo de chat, llamamos al metodo `join` para
crear a un nuevo receiver en el canal. Entonces este spawnmea una nueva tarea
asincrona para monitorear la rececion de mensajes y escribir de nuevo al cliente
en la funcion `handle_subscribe`

Con estos detalles en mano, el metodo `Group::post` es sencillo: este
simplemente envia mensaje a el canal que hace el broadcast. Dado que los valores
llevados por el channel son `Arc<String>` dado que cada receptor recibe su
propia copia de los mensajes este solo incrementa el contador sin hacer ninguna
copia!!! o allocation de memoria. Una vez que todos los subscribers han
transmitido el mensaje, el contador de la referencia cae a zero y el mesaje es
liberado(osea el `String`)

Esta es la definicion de el `handle_subscriber`:

```rust
use async_chat::FromServer;
use tokio::sync::broadcast::error::RecvError;

async fn handle_subscriber(group_name: Arc<String>,
                           mut receiver: broadcast::Receiver<Arc<String>>,
                           outbound: Arc<Outbound>)
{
    loop {
        let packet = match receiver.recv().await {
            Ok(message) => FromServer::Message {
                group_name: group_name.clone(),
                message: message.clone(),
            },

            Err(RecvError::Lagged(n)) => FromServer::Error(
                format!("Dropped {} messages from {}.", n, group_name)
            ),

            Err(RecvError::Closed) => break,
        };

        if outbound.send(packet).await.is_err() {
            break;
        }
    }
}
```

Aunque los detalles son diferentes, la forma de esta funcion es familiar: es un
loop que recibe mensajes desde el canal de broadcast y los transmite de nuevo al
cliente via un valor compartido `Outbound`. Si el loop no puede seguir con el
canal de broadcast, este recibe un error de `Lagegd`, el cual obedientemente
reporta al cliente. Si enviar el packet de nuevo al cliente falla completamente,
quizas porque la conexion se ha cerrado, `handle_subscriber` sale de ese loop y
retorna, causando que la tarea asincronica haga un exit. Esto tira el canal de
broadcast Receiver desuscribiendolo del canal. Esta manera cuando una conexion
es tirada cada uno de los grupos de membresia es limpiado la proxima vez que el
grupo intente enviar un mensaje

### Los types primitivos `Future` y `Executor`: Cuando volver a consultar?

Ahora que vimos como se usan las primitivas de este framework podemos ver como
estan implementadas. La pregunta principal es cuando un future retorna un
`Poll::Pending`, como este se coordina con el executor para hacer un poll de
nuevo en el tiempo correcto. Pensemos que pasa con codigo como el que sigue:

```rust
task::block_on(async {
    let socket = net::TcpStream::connect(address).await?;
    //...
})
```

La primera vez que `block_on` hace un poll sobre ese bloque async que esta
manejado por la conexion TCP, la red no estara lista inmediatamente por ello
`block_on` se pondra en sleep, pero cuando este deberia despertarse???. El
`TcpStream` tiene que de alguna forma comunicarselo a `block_on` que deberia
tratar de hacer un poll de nuevo

Cuando un executor como `block_on` poolea un future este debe ser pasado en un
callback llamado el `waker`. Si el future no esta listo entonces deberiamos
retornar un `Poll::Pending` por ahora y arreglar para que el `waker` que sea
invocado despues. Entones una implementacion de un `Future` a menudo luce como
esto:

```rust
use std::task::Waker;

struct MyPrimitiveFuture {
    //...
    waker: Option<Waker>,
}

impl Future for MyPrimitiveFuture {
    type Ouput = asdfasd;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<...> {
        // ...
        if future is ready {
            return Poll::Ready(final_value);
        }

        // save the waker for later
        self.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}
```
En otras palabras si el future esta listo lo retornamos. De otra manera
reservamos un clone del waker `Context` en algun lado y retornamos un
`Poll::Pending`. Como el future va a ser pooleado de nuevo, el future debe
notificar al ultimo executor que debe llamar a su `waker`:

```rust
// si tenemos un waker, invoquelo y limpie self.waker
if let Some(waker) = self.waker.take() {
    waker.wake();
}
```
Los futures de funciones async y bloques async no tratan con los waker por si
mismos sino que pasan el contexto que le han sido pasado a los subfutures que
ellos estan awaiting, delegando asi la obligacion de salvar y invocar al waker

`Waker` implementa `Clone` y `Send`, entonces un future puede siempre hacer su
propia copia de un waker y enviarlo a otros threads. Como vemos el metodo
`Waker::wake` consume el waker, pero hay tambien otro llamado `wake_by_ref` que
no lo hace, pero algunos executors pueden implementar la version que consume un
poco mas eficiente


### Invocando wakers: `spawn_blocking`

Antes en este capitulo vimos la funcion `spawn_blocking` el cual ponia en una
thread exclusiva una computacion costosa haciendo que el SO se encargue de las
optimizaciones para que no repercurta en la carga del sistema. Podemos ahora
implementar nuestra version de esta, por simplicidad, nuestra version crea un
thread por cada closure en lugar de usar un pool de threads como la version de
`async_std` hace

En lugar de retorne un `Future` lo que vamos a hacer es una simple funcion que
retorne una `struct` llamada `SpawnBlocking` sobre la cual vamos a impl el trait
`Future`. Donde la firma de la funcion es la siguiente:

```rust
pub fn spawn_blocking<T, F>(closure: F) -> SpawnBlocking<T>
where F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
```

Dado que necesitamos enviar el closure a otro thread y obtener el resultado de
vuelta, tanto el closure `F` como su valor de retorno `T` deben impl `Send` y
como no tenemos idea de cuanto va a tardar en correr el thread entonces tenemos
que darle un lifetime de `'static`, estos son los mismos limites que
`std::thread::spawn` impone

`SpawnBlocking<T>` es un future del valor de retorno del closure. Veamos su
definicion

```rust
use std::sync::{Arc, Mutex};
use std::task::Waker;

pub struct SpawnBlocking<T>(Arc<Mutex<Shared<T>>>);

struct Shared<T> {
    value: Option<T>,
    waker: Option<Waker>,
}
```

El struct `Shared` sirve como punto de encuentro entre el future y el thread que
corre el closure, por lo que tiene la propiedad con un `Arc` y protegido con un
`Mutex`(un mutex sincronico esta bien aqui). Polleando el future chequea cuando
un valor esta presente y guarda el waker en `waker`. El thread que corre el
closure guarda su valor de retorno en un valor y entonces invoca a waker si es
que esta presente. Aqui la definicion completa de `spawn_blocking`:

```rust
pub fn spawn_blocking<T, F>(closure: F) -> SpawnBlocking<T>
where: F: impl FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let inner = Arc::new(Mutex::new(Shared{
        value: None,
        waker: None,
    }));

    std::thead::spawn({
        let inner = inner.clone();
        move || {
            let value = closure();
            let maybe_waker = {
                let mut guard = inner.lock().unwrap();
                guard.value = Some(value);
                guard.waker.take()
            };
            if let Some(waker) = maybe_waker {
                waker.wake();
            }
        }
    });
    SpawnBlocking(inner)
}
```

Despues de crear el valor de `Shared`, esto spawmea un thread para correr el
closure guardar el resultado en el campo del `Shared` llamado `value` e invocar
a waker si es que hay alguno...

Podemos impl `Future` para `SpawnBlocking` de la siguiente manera:

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

impl<T: Send> Future for SpawnBlocking<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let mut guard = self.0.lock().unwrap();
        if let Some(value) = guard.value.take() {
            return Poll::Ready(value);
        }
    }

    guard.waker = Some(cx.waker().clone());
    Poll::Pending
}
```

Como vemos Polling un `SpawnBlocking` chequea si el value del closure esta
ready, tomando propiedad del mismo y retornando si es que eso pasa. De otra
manera el future continua en pendiente entonces guardamos un clone del waker que
esta en el `Context`

### "Pinning"

Aunque las funciones async y los bloques son escenciales para escribir codigo
claro asincrono, manejar esos futures requiere un poco de cuidado. El type `Pin`
nos ayuda a asegurar que seran usados de manera segura

Sirve como sello de aprobacion sobre los punteros que manejan futures

### Las dos stadios de vida de un `Future`

Consideremos esta simple funcion asicrona

```rust
use async_std::io::prelude::*;
use async_std::{io, net};

async fn fetch_string(address: &str) -> io::Result<String> {
    (1)
    let mut socket = net::TcpStream::connect(address).await(2)?;
    let mut buf = String::new();
    socket.read_to_string(&mut buf).await(3)?;
    Ok(buf)
}
```

Esto abre una conexion TCP en la direccion dada y retorna como `String` lo que
quiere retornar el server. Los puntos que señalamos como (1), (2) y (3) son los
puntos de reanudacion, los puntos en la funcion asincrona en donde la ejecucion
puede suspenderse

Supongamos que la llamamos sin hacer un awaiting, como por ejemplo:

```rust
let response = fetch_string("localhost:6502");
```

Ahora response es un future listo para comenzar la ejecucion en el comienzo de
`fetch_string` con el argumento dado. Dado que creamos este future y la
ejecucion deberia comenzar en el punto (1) en el principio del cuerpo de la
funcion. En este estado los unicos valores a futuro que necesitamos para seguir
son los argumentos de la funcion. Ahora supongamos que polleamos la respoonse
varias veces y llegamos a este punto en el cuerpo de la funcion:

```rust
socket.read_to_string(&mut buf).await(3)?;
```

Y Supongamos que el resultado de `read_to_string` no esta listo entonces el poll
retorna un `Poll::Pending`. Un future debe siempre guardar la informacion
necesaria para resumir la ejecucion la proxima vez que es polleada. En este
caso:

 - La reanudacion en el punto (3) diciendo que la ejecucion debe resumir en el
   await del future que devuelve `read_to_string`
 - Las variables que estan vivas en el punto de reanudacion: `socket` y `buf`.
   El valor de direccion no esta mas presente en el future dado que no se
   necesita mas
 - El subfuture `read_to_string` el cual la expresion await esta

<!-- NOTE(elsuizo: 2023-05-30): falta un poco mas de esto pero es mas para
leerlo que para resumirlo-->

### Pinned pointers

El type `Pin` es un wrapper para punteros a futures que restringe como los
punteros pueden ser usados para asegurarse de que los futures no pueden moverse
una vez que ellos han sido poolleados. Estas restricciones pueden ser sacadas
para futures que no importa sin son movidas, pero son escenciales para pollear
futures de manera segura de funciones async y bloques. Por punteros queremos
decir cualquier type que implemente `Deref` y posiblemente `DerefMut`. Un `Pin`
que envuelve un puntero se lo conoce como "pinned pointer" `Pin<&mut T>` y
`Pin<Box<T>>` son los mas comunes. La definicion de `Pin` en la libreria
estandar es simple:

```rust
pub struct Pin<P> {
    pointer: P
}
```

Notemos que el campo `pointer` es privado. Esto quiere decir que la unica manera
de construir o usar un `Pin` es eligiendo cuidadosamente los metodos que el type
provee

Dado un future de una funcion asincronica o bloque, hay solo algunas pocas
maneras de obtener un puntero a el:

 - el macro `pin!` desde el crate `futures-lite` enmascara una variable de type
`T` con uno nuevo de type `Pin<&mut T>`. La nueva variable apunta al valor
 original, el cual ha sido movido a una locacion temporaria anonima en la stack.
Cuando la variable se va fuera del scope, el valor es dropeado. Usamos `pin!` en
nuestro implementacion de `block_on` para pinnear un future que queremos pollear

 - El constructor de la libreria estandar `Box::pin` toma la propiedad del valor
 de cualquier type `T`, lo mueve a la heap y retorna un `Pin<Box<T>>`

 - `Pin<Box<T>>` implementa `From<Box<T>>` entonces `Pin::from(boxed)` toma
 propiedad del valor boxeado y nos devuelve un box pinneado que apunta al mismo
`T` en el heap


### Cuando es el codigo async valioso???

Codigo async es mas dificil de escribir que codigo para multithread, porque
tenemos que usar las primitivas de sincronizacion adecuadas para I/O separar
codigo que tiene mucha complejidad computacional a mano o hacer un spin a otras
threads y manejar otas cosas como pinning que en multithreading no tenemos que
preocuparnos. Entonces cual es la principal ventaja de codigo async???

Dos de las creencias de async que vamos a desmitificar son:

 - "Codigo async es bueno para I/O": Esto no es del todo correcto si tu
 aplicacoin esta perdiendo el tiempo esperando por I/O, hacerlo async no lo hara
mas rapido. No hay nada acerca de la interfaz I/O en async que lo haga mas
 rapido. El sistema operativo tiene que hacer el mismo trabajo de cualquiera de
las dos formas(de hecho en operaciones async I/O que no esta lista y tiene que
 probar mas tarde, entonces tomara dos llamadas al sistema para completar en
 lugar de una)

 - "Codigo async es mas facil de escribir que multithread": En lenguajes como
 Python o javascript puede ser pero en Rust no. Pero la ventaja de Rust es que
 una vez que compila tenemos certeza de que no tendremos data-races.


Entonces cuales son las ventajas de async?:

 - Las tareas async consumen mucha menos memoria
 - Las tareas async son mas rapidas para ser creadas, por ejemplo en linux crear
una thread lleva aprox: `15us` y spawmear una async `30ns`
 - Los cambios de contexto son mas rapidos entre tareas async que entre threads
del sistema operativo

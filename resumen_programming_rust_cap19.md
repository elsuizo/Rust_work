# Cap: 19: Concurrency

A lo largo de los años los programadores han desarrollado un vocabulario en
cuanto a programar sistemas concurrentes, los mas comunes son:

   - Un *tread* en el background que tiene un solo trabajo y se despierta
     periodicamente para hacerlo

   - Pools de proposito general que comunican con los clientes via queues de
     tareas

   - Pipelines donde los datos fluyen de un thread hacia el proximo, donde cada
     thread hace un minimo esfuerzo

   - Paralelismo de los datos: donde se asume(equivocadamente o no) que toda la
     computadora esta haciendo solo una gran tarea de computacion, la cual se
     divide en n partes y cada una de esas n partes se corre en un thread, con
     la esperanza de que se pongan todos los n cores de la maquina a funcionar

   - Un mar de objetos sincronizados: donde multiples thrads tienen acceso a la
     misma data, y los data races son evitados usando esquemas de "hoc locking"
     basados sobre primitivas de bajo nivel como los mutex(Java incluye este
     modelo que fue muy popular en los 90 - 2000s)

   - Operaciones enteras atomicas: permiten que muchos cores se comuniquen
     pasando informacion a traves de los campos que tienen un size de una
     palabra de maquina (esto es muy raro de utilizar lo que se utiliza en
     cambio son punteros en la practica)


Rust ofrece una mejor manera de usar concurrencia, no forzando a todos los
programas a que adopten un estilo en particular(lo cual para progrmacion de
sistemas no seria una solucion) pero si soportando muchos estilos de manera
segura. Las reglas que no se explicitan (en codigo) y son forzadas por el
compilador


## Fork-Join paralelismo

El caso de uso mas simple para threads es cuando tenemos tareas completamente
independientes que queremos hacerlas a todas de una sola vez. Por ejemplo
supongamos que estamos haciendo procesamiento de lenguaje natural en un
documento que es bastante largo. Podriamos escribir el siguiente loop:

```rust
fn process_file(filenames: Vec<String>) -> io::Result<()> {
   for document in filenames {
      let text    = load(&document)?; // read the source file
      let results = process(text);    // compute statistics
      save(&document, results)?;      // write output file
   }
   Ok(())
}
```
ese programa en un single thread va a ser algo como lo siguiente:

load ---> process ---> save ---> load ---> process ---> save ---> load ...

Ya que cada documento va a ser procesado separadamente, es relativamente facil
incrementar el rendimiento separando las tareas en pedazos y procesando cada
uno de estos pedazos en un thread separado. Este patron de procesamiento se
llama "fork-join parallelism". Lo de fork es porque por cada nuevo empezamos un
nuevo thread y el join es porque tenemos que esperar a que finalice. Ya hemos
visto esta tecnica en el ejemplo de `mandelbrot` del capitulo 2

"Fork-join" es atractivo por las siguientes razones:

 - Es muy simple de implementar y Rust hace que lo hagamos de manera coreecta

 - Evita los cuellos de botella, ya que no hay locking de recursos. La unica
   manera de que una thread tenga que esperar por otra thread es al final,
   mientras cada thread puede correr libremente. Esto ayuda a mantener el
   overhead cuando cambiamos de tareas bajo.

 - Es facil de razonar cuando queremos saber si nuestro programa es correcto,
   ya que un "fork-join" es deterministico ya que los threads estan aislados.
   El programa siempre produce el mismo resultado sin importar la variacion en
   la velocidad de los threads, es un modelo de concurrencia que no tiene
   "data-races"

La principal desventaja de este metodo es que requiere que las unidades de
trabajo esten aisladas una de otra, o sea que hay problemas que no se pueden
separar tan facilmente sus tareas


### `spawn` y `join`

La funcion de la libreria estandar: `std::thread::spawn` comienza un thread
nuevo:

```rust
spawn(|| {
   println!("hello from a child thread!!!");
})
```

Toma un argumento, que es un closure o funcion del type: `FnOnce`, Rust
comienza un nuevo thread para correr el codigo de ese closure o funcion. El
thread nuevo es un thread real del SO que tiene su propio stack, como los
threads de C++ o Java Veamos un ejemplo mas que usa esta funcion para
implementar la version en paralelo del procesador de files que vimos antes:

```rust
use std::thread::spawn;

fn process_files_in_parallel(filenames: Vec<String>) -> io::Result<()> {
   // divide the work into several chuncks
   const NTHREADS: usize = 8;
   let worklists = split_vec_into_chuncks(filenames, NTHREADS);

   let mut thread_handles = vec![];

   // fork: Spawn a thread to handle each chunck
   for worklist in worklists {
      thread_handles.push(spawn(move || process_files(worklist)));
   }

   // join: Wait for all threads to finish
   for handle in thread_handles {
      handle.join().unwrap()?;
   }

   Ok(())
}
```

Como vemos tenemos la misma signatura de la primer funcion, lo que hace que sea
mas facil reemplazarla. Utilizamos la funcion utilitaria
`split_vev_into_chuncks()` para dividir el trabajo, cuyo resultado `worklists`
es un vector de vectors, que contiene 8 slices que tienen un size igual al
vector de entrada, luego spawmeamos un thread por cada uno de los worklist.
Aqui `spawn()` retorna un `handle` para cada uno de los threads, que los
guardamos en un vector para usarlos luego.  Notemos como manejamos la lista de
filenames en un worked thread:

 - La worklist es definido y rellenado por el loop for, en el thread pariente

 - Ni bien que el closure es creado, `worklist` es movido dentro del closure

 - `spawn` entonces mueve el closure(incluyendo el vector worklist) a el nuevo
   thread hijo


Estos `move` son baratos en el sentido de memoria, ya que un `Vec<String>` los
`String`s no son clonados, de hecho nada es allocado o liberado. Los unicos
datos que son movidos son el `Vec<>` en si mismo que son 3 codigo de maquina
nada mas (el puntero donde empieza, el lenght, y el tipo de datos creo) Usamos
el metodo `.join()` del type `JoinHandles` que colectamos anteriormente para
esperar a los `NTHREADS` a que terminen. Esto de "joining" es importante porque
un programa de Rust termina ni bien el `main()` retorna aun si hay threads
corriendo. Los detructores no son llamados y los threads extra son asesinados


### Manejo de errores a lo largo de los threads

El codigo que utilizamos para hacer el "join" del thread hijo en el ejemplo
enterior es un poco raro, ya que es un `unwrap()` con seguido del operador `?`
El methodo `join()` hace dos cosas fantasticas por nosotros.

 - Primero: `handle.join()` retorna un `std::thread::Result` que es un error si
   el thread hijo paniquea. Esto hace que Rust sea dramaticamente mas robusto
   que `C` o `C++` como sabemos un "out-of-bounds" en arrays es un UB y no se
   proteje al resto del sistema de las consecuencias que ello trae. En cambio
   en Rust, panic es seguro y ademas es por cada thread que tenemos. Podemos
   recuperarnos de un error que se produce en un thread, un panic que sucede en
   un thread no se expande a los threads que dependen de el sino que un panic
   que sucede en un thread es reportado como un `Result::Err(e)` en los otros
   threads, asi el programa como un todo se puede recuperar facilmente de ese
   error.
   En nuestro ejemplo no hacemos nada de eso sino que usamos un `unwrap()` de
   ese `Result` afirmando que este `Result` sera un `Ok` y no un `Err`. Si el
   thread hijo paniquea entonces esto no es mas cierto y el thread padre va a
   paniquear tambien. Osea que estamos propagando explicitamente los panics del
   hijo hacia los niveles superiores

 - Segundo: `handle.join()` pasa el valor retornado desde el thread hijo hacia
   el padre. El closure que le pasamos a `spawn()` tiene como type de retorno
   `io::Result<()>` porque esto es lo que `process_file()` retorna. Este valor
   de retorno no es descartado. Cuando el thread hijo es finalizado, su valor
   de retorno es guardado y `JoinHandle::join()` transfiere el valor de vuelta
   hacia los threads padres. El type completo que retorna `handle.join()` en
   este programa es `std::thread::Result<std::io::Result<()>>`. La parte
   `std::thread::Result<>` es de la API de `spawn` y `join` y
   `std::io::Result<>` es parte de nuestra app. En nuestro caso despues de
   hacer el `unwrap()` usamos el operador `?` sobre el `std::io::Result<>` para
   propagar los errores de I/O explicitamente desde el hijo hacia los padres. Y
   todo esto pasa en una sola linea de codigo!!!


### Compartiendo datos inmutables a lo largo de "threads"

Supongamos que el analisis que estamos haciendo requiere una base de datos de
las palabras que hay en el idioma ingles:

```rust
// before
fn process_files(filenames: Vec<String>)

// after
fn process_files(filenames: Vec<String>, glosary: &GigabyteMap)
```

Osea que queremos pasarle esta referencia de la base de datos a cada uno de los
threads, podemos pensar en cambiar el codigo de la manera obvia como sigue:

```rust
fn process_files(filenames: Vec<String>, glosary: &GigabyteMap) -> io::Result<()> {
   // ...

   for worklist in worklists {
      thread_handles.push(spawn(move || process_files(worklist, glosary))); // error!!!
   }
}

```

Lo que simplemente hicimos es agregar el parametro a la funcion pero no podemos
compartir una referencia a lo largo de mas de un thread de esta manera, ya que
Rust no tiene manera de saber cuanto va a correr el thread hijo, entonces asume
el peor caso posible. Pero por suerte la libreria estandar tiene un type
especialmente para esto que es el type `Arc`

Ahora nuestro ejemplo queda de la siguiente manera:

```rust
use std::sync::Arc;

fn process_files_in_parellel(filenames: Vec<String>, glosary: Arc<GygabyteMap>) -> io::Result<()> {
   // ..

   for worklist in worklists {
      // this call to .clone() only clones the Arc and bumps the reference count
      // It dont clone the GygabyteMap!!!
      let glosary_for_childs = glosary.clone();
      spawn(move || process_files(worklist, &glosary_for_childs));
   }
}
```

Lo que hemos cambiado es el type de `glosary`: para hacer el analisis en
paralelo el que llama a la funcion de pasarle un `Arch<GigabybteMap>` que es un
"smart pointer" a el type `GygabyteMap` que ha sido movido al heap haciendo un
`Arc::new(giga_map)` Cuando llamamos al metodo `glosary.clone()` estamos
haciendo una copy del puntero en si no de toda la data que hay en el (seria un
desastre!!!). Con este camnbio el programa compila ya que no depende mas de una
referencia con lifetime. Mientras haya un thread que sea duenio de el
`Arc<GigabyteMap>` este se mantendra vivo, aun si el padre se "muere" mas
rapido que este. No habra ninguna "data race" ya los datos en un `Arc` son
inmutables


### Rayon

La funcion de la libreria estandar `spawn()` es muy importante en el contexto
de threads, pero no esta diseniada especificamente para hacer un "fork-join".
Se han construido mejores APIs para estos metodos de paralelismo. Por ejemplo
en el capitulo 2 vimos que se puede utilizar la libreria `Crossbeam` para hacer
un "split" del trabajo a lo largo de 8 threads. `Crossbeam` soporta el metodo
de paralelismo "fork-join" de manera natural. La libreria `Rayon` es otro
ejemplo. Provee dos maneras de correr tareas concurrentemente:

```rust
extern crate rayon;
use rayon::prelude::*;

// do 2 things in parallel
let (v1, v2) = rayon::join(fn1, fn2);

// do N things in parallel
giant_vector.par_iter().for_each(|value| {
   do_thing_whith_value(value);
})
```

En la primera manera solo llamamos a las funciones `fn1` y `fn2` y retornamos
el ambos `Result<>`s. En la otra version el metodo `par_iter()` crea un iterador
`ParallelIterator` que tiene las funciones historicas de cualquier iter como `map`
`filter` y otros metodos. Lo bueno es que la libreria se encarga de distribuir el
trabajo por nosotros. Ahora veamos la version de nuestra funcion pero usando
rayon


```rust
extern crate rayon;
use rayon::prelude::*;

fn process_files_in_parallel(filenames: Vec<String>, glosary: &GigabyteMap) -> io::Result<()> {
   filenames.par_iter()
                     .map(|filename| process_file(filename, glosary))
                     .reduce_with(|r1, r2| {
                        if r1.is_err() {r1} else {r2}
                     })
                     .unwrap_or(Ok())
}
```


### Channels

Un channel es ruta de un sentido para enviar valores desde un thread hacia otro
En otras palabras es una queue que es segura en el contexto de los threads

Son algo asi como los pipes de Unix.

`sender.send(item)`: pone un simple valor en un `channel`, `receiver.recv()`:
remueve un valor de la queue. La propiedad es transferida desde la thread que
envia a la que recibe. Si el canal esta vacio, `receiver.recv()` bloquea hasta
que el valor se haya enviado. Con `channels` los threads se pueden comunicar
pasandose valores desde uno al otro. Es un esquema muy sencillo para que los
threads trabajen juntos sin usar un esquema que bloquee o comparta memoria

Los "channels" de Rust son mas rapidos que los pipes de Unix, ya que enviar un
valor lo mueve en lugar de copiarlo y mover es mucho mas rapido que copiar aun
cuando estas tratando con estructuras de datos con megabytes de tamanio


### Enviando valores

Vamos a construir un programa concurrente que crea un indice invertido(???)

https://en.wikipedia.org/wiki/Inverted_index

que es uno de los ingredientes principales de un buscador. Todos los buscadores
funcionan sobre una colleccion de documentos. Este indice es la base de datos
que nos dice cual palabra va a aparecer en donde. La totalidad del codigo esta
en el repo: `fingertips`, que esta estructurado como un "pipeline" que son una
de las tantas maneras de usar `channels` y son una de las maneras mas sencillas
de introducir concurrencia en un programa que tiene un solo thread

Usamos un total de 5 threads, cada una de las cuales realiza una tarea dintinta
cada thread produce salidas continuamente en el lifetime del programa. El primer
thread por ejemplo, simplemente lee el contenido del documento desde el disco a
memoria, uno por uno (queremos un thread que haga esto porque estamos haciendo
el codigo mas simple posible, usando `std::File::open` y `read_to_string()` los
cuales son bloqueantes). La salida de este estado del pipeline es un gran `String`
por documento, entonces este thread es conectado con el proximo thread por un
`channel` de `String`s

Nuestro programa comienza por el thread que lee los archivos, supongamos que estos
son un `Vec<PathBuf>` un vector de lifetimes. El codigo que inicia al thread de
lectura se ve algo asi:

```rust
use std::fs::File;
use std::io::prelude::*; // for Read::read_to_string
use std::thread::spawn;
use std::sync::mpsc::channel;

let (sender, receiver) = channel();

let handle = spawn(move || {
      for filename in documents {
         let mut f = File::open(filename)?;
         let mut text = String::new();
         f.read_to_string(&mut text)?;

         if sender.send(text).is_err() {
            break;
         }
      }
      Ok(())
      });
```

La funcion `channel()` retorna un par de valores un "sender" y un "receiver".
Los channels tienen su propio type, osea que si queremos usar este `channel`
para enviar el texto de cada file tenenmos que tener un "sender" del type:
`Sender<String>` y un "receiver" de type: `Receiver<String>` Despues de una
lectura existosa del file enviamos el texto por el `channel` con el metodo
`send()`. Asi el texto contenga 10 lineas de texto o 1000 Gb cuando movemos el
mismo al `channel` lo unico que estamos haciendo es copiar tres palabras de
codigo esas tres palabras de codigo de maquina tambien. Los dos metodos
retornan un `Result<>` de maquina (que es el size del String) y el
correspondiente `receiver.recv()` copia pero estos metodos fallan solo cuando
el otro "extremo" del `channel` ha sido tirado Osea que un `send` falla cuando
un `Receiver` ha sido tirado

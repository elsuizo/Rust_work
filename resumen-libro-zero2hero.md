# Resumen del libro: "Zero to hero" de Luca Palmieri

## Construyendo un newsletter para email

Lo que hace en este libro es tomar un projecto practico e ir haciendo todos los
pasos para su realizacion

## Sign up y un nuevo subscriptor

En este primer approach vamos a implementar la siguiente experiencia de usuario:

```text
Como visitador del blog
Quiero Subcribirme a el newsletter
Para que pueda recibir updates al mail cuando un nuevo contenido es publicado
```

Esperamos que el visitador del blog ingrese el mail en un formulario que esta
emebebido en una pagina web, el formulario va a triggear una llamada a la API
que esta implementada en un server de backend que procesa la informacion, guarda
la informacion y envia una respuesta

Este capitulo se enfoca en ese server que implementara los endpoints
`subscriptions/POST`

### Nuestra estrategia

Como estamos comenzando un proyecto desde cero tenemos que decidir las
siguientes items:

 - Elegir un framework web y familiarizaros con el
 - Definir la estrategia de testing
 - Elegir un crate para interactuar con nuestra base de datos(tenemos que
   guardar esos mails en algun lugar!!!)
 - Definir como queremos manejar los cambios en la base de datos a traves del
   tiempo(conocida como migraciones)
 - Escribir algunos querys piolas

### Eligiendo el web framework

El autor dice que hasta marzo de 2022 el mejor para el es `actix-web`

Es una buena idea tener a mano los siguientes enlaces:

[actix-web](https://actix.rs/)

[actix-web-docs](https://docs.rs/actix-web/4.0.1/actix_web/index.html)

[actix-web-examples](https://github.com/actix/examples)

### Nuestro primer end-point

Vamos a implementar lo que seria el hola mundo endpoint: cuando recibimos un
request `GET` para `health_check` vamos a retornar `200 OK` como respuesta sin
ningun body

Podemos usar `/health_check` para verificar que la aplicacion esta corriendo y
lista para aceptar request. Combinando eso con el servicio SaaS Como

[pingdom.com](pingdom.com) y podemos ser alertados cuando la API no va bien (que
es una muy buena plataforma de comienzo para el newsletter)

Otro uso del `/health_check` podria ser si utilizamos un container orquestador
para hacer malabares con la aplicacion (por ejemplo Kubernetes o Nomad). El
orquestador puede llamar a `/health_check` para detectar si la API se ha quedado
sin respuestas y asi triggear un reset


### Escribiendo con `actic-web`

Nuestro punto de partida va a ser el hola mundo de la pagina de actix

```rust
use actix_web::{web, App, HttpRequest, HttpServer, Responder};

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}", &name)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
```

### Anatomia de una aplicacion con actix-web

### server - `HttpServer`

Es el esqueleto de nuestra aplicacion, tiene en cuenta las siguientes cosas:

 - Donde la aplicacion va a escuchar las request que vienen? Un socket TCP
   (ejemplo `127.0.0.1:8000`) ? Un dominio Unix?
 - Cual es el maximo numero de conexiones concurrentes que deberiamos permitir?
   Cuantas nuevas conexiones por unidad de tiempo???
 - Deberiamos habilitar TLS (Transport Layer Security)???

En otras palabras `HttpServer` tiene en cuenta todos los niveles de transporte.

Pero que pasa despues de eso? Que hace `HttpServer` cuando tiene una conexion
establecida con un cliente de nuestra API y necesitamos comenzar a manejar las
request de el???

Aqui es donde `App` entra en juego


### Aplicacion `App`

Es donde todas las logica de las aplicaciones viven: routing, middlewares,
requests, etc..

`App` es el componente en el cual toma una request como input y emite una
response como salida

En nuestro codigo seria en la parte:

```rust
App::new()
     .route("/", web::get().to(greet))
     .route("/{name}", web::get().to(greet))
```

Vemos que el type `App` es un claro ejemplo de el patron `Builder`


### End point `Route`

Como agregamos un endpoint a nuestra `App` ???

El metodo `route` es probablemente la manera mas simple de hacerlo, este toma
dos parametros:

 - `path`: un string que posiblemente puede venir de un template (ej: "/{name}")
   para acomodar paths dinamicos
 - `route`: una instancia de la struct `Route`

`Route` combina un handler con un conjunto de *guards*

Los **Guards** especifican las condiciones para las que un request debe
satisfacer para "matchear" y ser pasadas sobre el handler. Desde una perspectiva
de implementacion los **guards** son las implementaciones de el trait `Guard`

`Guard::check` es donde la magia sucede

En el snippet de codigo tenemos:

`route("/", web::get().to(greet))`

Y como se ve el handler ??? Como es la firma de su funcion. Tenemos un solo
ejemplo por ahora `greet`

```rust
async fn greet(req: HttpRequest) -> impl Responder {
  // ...
}
```

Pero tienen que tener todas las funciones de handler la misma firma??? No!!!

porque `actix_web` con unos trucos con traits nos permite un rango de diferentes
firmas para funciones, especialmente con los argumentos de entrada

### Runtime `tokio`

Necesitamos que el main sea asincronico porque `HttpServer::run` es un metodo
asincronico. La programacion asincronica en Rust es construida sobre el trait
`Future`. Todos los types que implementan este trait exponen el metodo `poll` el
cual tiene que ser llamado para permitir que el **Future** progrese y
eventualmente resuelva su valor final

### Implementando el `Healt Check Handler`

Ya tenemos las piezas principales en el hola mundo ahora podemos modificarlo
para obtener el `health_check` que retorna un `200 OK` como respuesta sin body
cuando recibimos un request `GET`

Imitando la funcion `greet` podemos plantear:

```rust
async fn health_check(req: HttpRequest) -> impl Responder {
    todo!()
}
```

Como dijimos que el trait `Responder` es solo un trait para hacer la conversion
hacia un `HttpResponse`, entonces si retornamos una instancia de ese type
directamente deberia funcionar

Si miramos la implementacion de `HttpResponseBuilder` vemos que tambien
implementa `Responder` y por ello podemos omitir nuestro llamado a `finish`

El proximo paso es manejar la registracion, necesitamos adicionarlo a nuestro
`App` via un `route`


### Nuestro primer test de integracion

`/health_check` fue nuestro primer endpoint y nosotros verificamos que todo
funciona como esperabamos con `curl`, como podemos imaginar cuando tenemos una
aplicacion mas grande no podemos hacerlo para cada uno de los endpoints
manualmente por eso necesitamos algun tipo de automatizacion


### Como testear un endpoint

Una API es un sinonimo de final: una herramienta que expone el mundo exterior
para hacer determinada tarea (por ejemplo guardar un libro, publicar un email)

El endpoint que expone nuestra API define un "contrato" entre nosotros y
nuestros clientes: un convenio compartido sobre cuales son las entradas y
salidas del sistema osea su interface

Siguiendo este principio, no queremos que las llamadas en las funciones de test
llamen a las funciones de handler directamente, por ejemplo:

```rust
#[cfg(test)]
mod tests {
    use crate::health_check;

    #[tokio::test]
    async fn health_check_succeeds() {
        let response = health_check().await;
        // this requieres changing the return type of `health_check`
        // from `impl Responder` to `HttpResponse` to compile
        // you also need to import it with `use actix_web::HttpResponse`
        assert!(response.status().is_succes())
    }
}
```

Esto no es buen test porque no testeamos si la funcion de handler es llamada
cuando en el `GET` recibe el path que esperamos. Como esperamos `actix_web`
provee muchas herramientas para hacer testing como podemos ver en su pagina:

[actix_web testing page](https://actix.rs/docs/testing/)

Pero como vemos vamos a tener problemas si por ejemplo queremos migrar a otro
framework web

Por eso optamos por una solucion que sea completamente una caja negra en el
sentido que vamos a lanzar la aplicacion en el comienzo de cada test y
interactuar con el usando un cliente HTTP (por ejemplo `reqwest`)

### Cambiando la estructura del proyecto para testear facilmente

Lo que hicimos es crear una carpeta nueva que la tenemos que llamar `tests` (por
una convencion de Rust) donde pusimos un nuevo archivo llamado `health_check.rs`
donde vamos a escribir los tests de la app

Luego agregamos una `lib` donde vamos a poner las implementaciones de la API y
seguimos con el main donde ponemos el entrypoint de la app


### Implementando nuestro primer test de integracion

Nuestro contrato para el `health_check` fue: "Cuando recibimos un GET para
/health_check retornamos un 200 OK de respuesta sin body"

Vamos a traducir esto en un test:

```rust
//! tests/healt_check.rs
//!
//! `tokio::test` es el equivalente de testeo para el tokio::main

// use zero2prod::run;

#[tokio::test]
async fn healt_check_works() {
    spawn_app().await.expect("Failed to spawn our app");
    let client = reqwest::Client::new();
    let response = client
        .get("http://127.0.0.1:8000/health_check")
        .send()
        .await
        .expect("Failed to executed");

    // assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// lanzamos la aplicacion en el backgroud de alguna manera
//
async fn spawn_app() -> std::io::Result<()> {
    zero2prod::run().await
}
```

Necesitamos correr nuestra app como una tarea en backgroud, por ello podemos
utilizar `tokio::spawn` para ello. Por ello vamos a hacer un refactor para que
`zero2prod::run` retorne un `Server` sin tener que esperar nada.

### Mejorando la app un poco

Que pasa con la app que esta corriendo en el backgroud cuando el test que esta
corriendo termina??? Este se "apaga"? sigue corriendo como zombie???

Una segunda mirada a la documentacion de `tokio::spawn` nos dice que cuando el
runtime de `tokio` se apaga todas las tareas que fueron spawnmeadas son
descartadas. Entonces `tokio::test` comienza un nuevo runtime en el comienzo de
cada test y "apaga" el finalizar el mismo. En otras palabras no necesitamos
hacer algun tipo de clean-up con el codigo de test


### Eligiendo un puerto random

 - Si el puerto 8000 es comenzado a utilizar por otro programa en nuestra
   maquina (por ejemplo esta misma aplicacion) el test va a fallar!!!
 - Si intentamos correr dos o mas tests en paralelo solo uno de ellos va a ser
   exitoso y los otros van a fallar

Podemos hacerlo mejor: los tests deberian correr en el backgroud sobre un puerto
random. Primero de todo necesitamos cambiar la funcion `run` esta deberia tomar
el address de la aplicacion como argumento en lugar de un valor hard-coded

Pero necesitamos saber cual es el port que el OS nos da para pasarselo al
`spawn_app` y hace. Hay muchas maneras de hacer esto una de ellas es con
`std::net::TcpListener`. Ya que nuestro `HttpServer` esta haciendo un trabajo
doble: dandonos una direccion el va a hacer el bind y luego comenzar la
aplicacion. Podemos tomar el primer paso: vamos a hacer el bind nosotros mismos
con el `TcpListener` y entonces entregar eso a el `HttpServer` usando `listen`

Cual es el lado positivo ???. `TcpListener::local_addr` retorna una `SocketAddr`
el cual expone el puerto actual en el que hicimos el bound via el metodo `.port`

Comencemos con la funcion `run`:

```rust
/// run method
pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().route("/health_check", web::get().to(health_check)))
        .listen(listener)?
        .run();

    Ok(server)
}
```


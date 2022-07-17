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

`spawn_app` siempre esta tratando de correr nuestra aplicacion sobre el puerto
8000 lo cual no es lo ideal:

 - Si el puerto esta siendo utilizado por otra aplicacion en nuestra maquina(por
   ejemplo nuestra propia aplicacion) el test deberia fallar
 - Si intentamos correr el test dos veces o mas o en paralelo solo uno de ellos
   sera capaz de bindear el port, los otros fallaran

Pero si solo cambiamos eso no tenemos meanera de saber cual es el puerto que nos
ha asignado el SO, como podemos solucionarlo???

Podemos utilizar `std::net::TcpListener` pero tambien existe una contra de este
metodo que es que el

```rust
async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}

/// run method
pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().route("/health_check", web::get().to(health_check)))
        .listen(listener)?
        .run();

    Ok(server)
}
```

```rust
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bin random port");
    // guardamos el port que nos ha asignado por el Sistema operativo
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Failed to bin address");
    // lanzamos el server como un proceso en el backgroud
    // tokio::spawn retorna un handle para spamear un Future
    // pero aca no lo usamos por ahora...
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
```

### Refocus

Como dijimos queremos que nuestro usuario ingrese su mail en un formulario html
sobre una pagina web. Este formulario va a lanzar una llamada
`POST/subscriptions` a nuestro API backend que es el encargado de procesar la
info, guardarla y enviar de vuelta una response

### Trabajando con formularios HTML

Lo que queremos es que por lo menos el usuario ingrese su nombre y su email para
poder con ell armar un minimo mensaje de bienvenida como hacen todos los
newsletters

Para ello primero tenemos que ver como codificar el body en el POST request.
Existen algunas pocas posibilidades proveniendo de un formulario de HTML, lo que
vamos a usar es la convecion que podemos ver en la MDN donde:

```text
application/x-www-form-urlencoded
```

Donde las keys y los values (en nuestro formulario) son codificados en una tupla
que esta separada por un `&` con un `=` entre medio del key y del value. Los
valores no alfabeticos son codificados con un `%` entre medio


Por ejemplo si el nombre es Legin y el mail es `ursula_le_guin@gmail.com` el
body del POST request sera: `name=le%20guin&email=ursula_le_guin%40gmail.com`
Donde vemos que los espacios son reemplazados por el `%20` y el `@` es
reemplazado por `%40`

Podemos ver de donde estan esas conversiones tabuladas en el siguiente enlace:

[tabla de URL Encoding](https://www.w3schools.com/tags/ref_urlencode.ASP)

Para recapitular:
 - si un valor de par de nombre y mail son enviados de manera
   correcta usando las convenciones de arriba el backend deberia retornar un `200
   OK`
 - Si algunas de las dos cosas que tiene que darnos el usuario estan mal
   entonces el backend deberia retornar un `400 BAD REQUEST`


### Capturando nuestros requerimientos como tests

Ahora que tenemos los requerimientos podemos plantear los test que necesitamos
para probar la API(osea lo que se llama test de integracion)

Hacemos un test para cada uno de los casos posibles de exito y de error cuando
el backend parsee los datos

```rust
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    // act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());
}
```

```rust
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let tests_cases = vec![
        ("name=le%20guin", "missing the mail"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in tests_cases {
        // act
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            // mensaje adicional que ponemos para que sea mas claro todo
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        );
    }
}
```

Ahora si corremos los tests van a fallar porque no tenemos Implementando
`/subscriptions`


### Parseando data desde una request POST

Lo que hizo aca es primero parsear desde el POST pero no anduvo uno de los tests


### Usando lo que nos ofrece `actic-web` (`extractors`)

Como el nombre lo indica `extractors` es una herramienta que nos da la libreria
para poder extraer info de las request que nos llegan, las mas utilizadas son:

 - `Path`: para obtener path dinamicos desde un path de request
 - `Query`: para parametros de Query
 - `Json`: para parsear bodys que tienen encodeados `.json`s

Por suerte tambien tenemos uno para lo que estamos utilizando:

[Form](https://docs.rs/actix-web/4.0.1/actix_web/web/struct.Form.html)

Pero como podemos utilizarlo???

Desde la pagina de la documentacion podemos leer:

```text
Un extractor puede ser accedido como argumento a una funcion handler. actic-web
soporta hasta 10 extractors por funcion de handler. Las posiciones de los
argumentos no importan
```

ejemplo:

```rust
use actix_web::web;
use serde::Deserialize;

#[derive(Deserialize)]
struct FormData {
    user_name: String,
}
// Extract form data using serde
// This handler get called only if content type is *x-www-form-urlencoded*
// and content of the request could be deserialized to a `FormData` struct
fn index(form: web::Form<FormData>) -> String {
    format!("Welcome {}!", form.user_name)
}
```

Entonces basicamente ponemos como argumento esa estructura de datos para nuestro
handler de `actic-web` cuando una request entra, que de alguna manera hace el
trabajo pesado por nosotros

Usande el ejemplo de como se utiliza a los extractors, podemos hacer algo asi:

```rust
#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
```

Y haciendo el import de `serde` que nos falta y corriendo los test ahora
funciona todo ok!!!

Pero porqueee???

<!-- TODO(elsuizo: 2022-07-16): aca corre el test y todo funciona por arte de
magia pero ami no me anda :( -->

### los traits `From` y `FromRequest`

Si vamos a el codigo fuente de estos vemos que es bastante simple:

```rust
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Form<T>(pub T);
```

Es nada mas que un wraper: y es generico sobre el type T el cual es usado para
rellenar el unico campo de `Form`

Pero ahi no es donde la magia sucede sino que en el trait `FromRequest` veamos
su definicion:

```rust
/// Trait implemented by types that can be extracted from requests
///
/// Types that implement this trait can be used with `Route` handlers
pub trait FromRequest: Sized {
    type Error = Into<actic-web::Error>;

    async fn from_request(req: &HttpRequest, payload: &mut Payload) ->
    Result<Self, Self::Error>;
}

/// omite algunos metodos mas que son auxiliares
```

Entonces como vemos la funcion mas importante lo que hace es tomar una
`HttpRequest` como argumento y los bytes de `Payload` y retorna un `Self` (que
seria el type que implemente el trait) si la extraccion tuvo exito o un error si
algo anduvo mal
Todos los argumentos en la firma de un **route handler** deben implementar el
trait `FromRequest` asi `actic-web` va a invocar `from_request` para cada
argumento y si la extraccion ha sido exitosa para todos entonces va a correr la
funcion actual de handler

Si alguna de las extracciones falla el error correspondiente es retornado a
donde fue llamado y el handler no es llamado nunca mas (tener en cuenta que los
errores de `actic-web` pueden ser convertidos a `HttpResponse`)

Esto es mut conveniente porque el handler no tiene que tratar con las requests
que vienen y pueden en cambio trabajar con informacion que viene de types y toda
su seguridad

Veamos ahora como es la implementacion de `From` para `FromRequest`:

```rust
impl<T> FromRequest for Form<T>
where
    T: DeserializeOwned + 'static,
{
    type Error = actix_web::Error;

    async fn from_request(req: &HttpRequest, payload: &mut Paylod) -> Result<Self, Self::Error> {

        // omitimos algunos detalles aca
        match UrlEncoded::new(req, payload).await {
            Ok(item) => Ok(Form(item)),
            // el error se puede customizar
            // el default es que retorne un 404, que es el que queremos que sea...
            Err(e) => Err(error_handler(e))
        }
    }
}
```

Como vemos casi todo el peso de la computacion lo lleva el type `UrlEncoded` que
es donde entra serde en todo esto porque comprime y descomprime **payloads**,
trata con el hecho de que el body de requests arriba un chunck por vez como
stream de bytes etc. Y la parte fundamental como dijimos tiene que ver con como
serde encode y decode la info por ejemplo en:

```rust
serde_urlencoded::from_byte::<T>(&body).map_err(|_| UrlEncodedError::Parse)
```

Que seria el lugar donde ocurre la deserializacion provieniente de serde


### Guardando datos: Base de datos

Como tenemos hasta ahora nuestro newsletter todavia no recolectamos ningun tipo
de informacion desde forularios de HTML. Para ello lo que se usa son las base de
datos


### Eligiendo una base de datos

Cual base de datos deberiamos elegir para nuestro proyecto???

El autor tiene como una regla para contestar esta pregunta y es la siguiente:

```text
Si no tenemos certeza sobre los requerimientos, use una base de datos
relacional. Si no tenemos dudas de que vamos a tener un escalamiento masivo
entonces use PostgreSQL
```

### Eligiendo un crate de database

En Agosto del 2020 (cuando escribio esto el autor debe ser...) los tres crates
mas importantes para interactuar con `PostgreSQL` en un projecto de Rust son:

 - `tokio-postgres`
 - `sqlx`
 - `diesel`

Como elegir uno de ellos???, bueno podemos separar sus diferencias en las
siguientes tres topicos:

 - seguridad en tiempo de compilacion
 - `SQL-first` vs `DSL` para construir las querys
 - `async` vs `sync`

#### Seguridad en tiempo de compilacion

Esta es una feature muy buena que nos permite el lenguaje ya que podemos
detectar un error en una query en tiempo de compilacion y no en tiempo de
ejecucion!!!, `diesel` y `sqlx` tienen ese feature

#### Interface de querys

Ambos `tokio-postgres` y `sqlx` esperan que nosotros usemos `SQL` directamente
para escribir los queries, `diesel` en cambio usa un DSL para esto


#### soporte para Async

Una de las frases que escucho el autor y que puede resumir lo que significa
`async` es la siguiente:

"Threads son para hacer trabajo en paralelo, async es para esperar en paralelo"

Ambos `sqlx` y `tokio-postgres` proveen una interface async, mientras que
`diesel` es sync y no tiene planes de cambiar en el futuro

#### Resumen

| Column1            | Compile time safety    | Query interface    | async |
|--------------------|------------------------|--------------------|-------|
| `tokio-postgres`   | No                     | SQL                | Si    |
| `sqlx`             | Si                     | SQL                | Si    |
| `diesel`           | Si                     | DSL                | No    |

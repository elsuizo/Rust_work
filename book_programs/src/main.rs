//-------------------------------------------------------------------------
//                        interior mutability
//-------------------------------------------------------------------------
// Supongamos que tenemos un robot que tiene una struct en la cual se ponen las variables de
// configuracion, esta se setea cuando bootea el robot y los valores nunca cambiaran en la vida del
// robot
pub struct SpiderRobot {
    species: String,
    web_anabled: bool,
    leg_devices: [fd::FileDesc; 8],
    ...
}
// luego todos los sistemas esenciales del robot es manejado por una struct y cada una tiene que
// "mirar" de nuevo hacia la struct SpiderRobot
use std::rc::Rc;

pub struct SpiderSenses {
    robot: Rc<SpiderRobot>, // <---- apunta a los settings de SpiderRobot
    eyes: [Camera; 32],
    motion: Accelerometer,
    ...
}
// Recordemos que Rc es una Reference counting y un valor que ponemos en esta "caja" sera siempre
// compartido y por ello siempre inmutable.
// Supungamos ahora que necesitamos que en SpiderRobot tenga dentro de ella un type mutable(como
// un File) lo que necesitamos entonces es solo un poco de data mutable dentro de una type que es
// inmutable. Rust ofrece dos posibilidades como Cell<T> y RefCell<T> ambas en el modulo std::cell
// Una Cell<T> es una struct que contiene una solo valor privado del type T. La unica cosa especial
// que tiene Cell<T> es que podemos setear el field aun cuando no tenemos acceso mut a la Cell
//
// Una Cell puede servir cuando por ejemplo tenenos un contador de la cantidad de errores que
// suceden en los distintos elementos del Hardware.
use std::cell::Cell;
pub struct SpiderRobot {
    ...,
    hardware_error_counter: Cell<u32>,
    ...,
}
// entonces todos los metodos podran incrementar y mirar ese valor desde "afuera" aun cuando sea
// inmutable SpiderRobot
impl SpiderRobot {
    // increase the error count by 1
    pub fn add_hardware_error(&self) {
        let n = self.hardware_error_counter.get();
        self.hardware_error_counter.set(n + 1);
    }

    // true if any hardware error have been reported
    pub fn has_hardware_errors(&self) -> bool {
        self.hardware_error_counter.get() > 0
    }
}
// Ahora lo que no podemos es con Cell llamar a metodos sobre un valor compartido. El metodo get()
// retorna una copia del valor en la Cell entonces solo funciona si T implementa Copy. Pero por
// ejemplo si necesitamos loggear un File no podremos porque un File no es Copyable. La herramienta
// que necesitamos utilizar es RefCell<T> que es un type generico que contiene un solo valor del
// type T, diferente a Cell, RefCell<T> soporta el prestamo de una referencia de nuestro valor T
// Los dos metodos paniquean si y solo si tratamos de romper la regla de Rust que dice que una
// referencia mut es exclusiva. Por ejemplo esto haria un panic:
let ref_cell: RefCell<String> = RefCell::new("hello".to_sting());
let r = ref_cell.borrow(); // ok, retorna una Ref<String>
let count = r.len();       // ok, retorna "hello".len()
assert_eq!(count, 5);

let mut w = ref_cell.borrow_mut(); // paniquea porque ya fue prestadaaa
w.push_str("world");

// entonce en nuestro SpiderRobot:
pub struct SpiderRobot {
    ...,
    log_file: RefCell<File>,
    ...,
}

impl SpiderRobot {
    // write a line to the log file
    pub fn log(&self, message: &str) {
        let mut file = self.log_file.borrow_mut();
        writeln!(file, "{}", message).unwrap();
    }
}
// tenemos que recordar que estos metodos no son posibles con multiples - threads

//-------------------------------------------------------------------------
//                        enums!!!
//-------------------------------------------------------------------------
// Los enums de Rust van mucho mas alla de los conocidos de C o C++ ya que estos pueden contener
// data que varie. C tiene union que se parece pero no es type-safe
// Tambien tenemos los viejos conocidos enums a la C:
// c-style enums
// Create a type Ordering con tres posibles valores llamado variantes o constructores
enum Ordering {
    Less,
    Equal,
    Greather
}

// como ya esta en la libreria estandar la podemos importar (asi no importamos todas las variantes)
use std::cmp::Ordering;
// podemos importar a todos los variantes con:
// use std::cmp::Ordering::*;

fn compare(n: i32, m: i32) -> Ordering {
    if n < m {
        Ordering::Less;
    } else if n > m {
        Ordering::Greater;
    } else {
        Ordering::Equal;
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum TimeUnit {
    Seconds, Minutes, Hours, Days, Months, Years
}

// los enums pueden tener metodos como las structs
//
impl TimeUnit {
    // return the plural noun for this time unit
    fn plural(self) -> &'static str {
        match self {
            TimeUnit::Seconds => "seconds",
            TimeUnit::Minutes => "minutes",
            TimeUnit::Hours   => "hours",
            TimeUnit::Days    => "days",
            TimeUnit::Months  => "months",
            TimeUnit::years   => "years"
        }
    }

    fn singular(self) -> &'static str {
        self.plural().trim_right_matches('s')
    }
}

// Los enums mas interesantes son los que tienen datos
// por ejemplo supongamos que queremos un programa que
// devuelva una frase aproximada con el tiempo que ha
// pasado
#[derive(Copy, Clone, Debug, PartialEq)]
enum RoughTime {
    InThePast(TimeUnit, u32),
    JustNow,
    InTheFuture(TimeUnit, u32)
}

// las dos variantes que toman argumentos se llaman tuple-variants. Como
// las structs estos constructores son funciones que crean nuevos valores
// del type RoughTime
// Los enums tambien puden tener structs-variants
enum Shape {
    Sphere{center: Point3D, radius: f32},
    Cuboid{corner1: Point3D, corner2: Point3D}
}

let unit_sphere = Shape::Sphere{center: ORIGIN, radius: 1.0};

// Todos las variantes de un enum pub son tambien pub

// con enums podemos hacer que nuestros programas sean muy verbosos
//
enum Json {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Box<HashMap<String, Json>>)
}

// podemos hacer estructuras de datos genericas muy facil!!!
//
// el ejemplo clasico de la libreria estandar es: Option<T>
enum Option<T> {
    None,
    Some(T)
}
// o tambien Result<T, E>
enum Result<T, E> {
    Ok(T),
    Err(E)
}
// Por ejemplo un arbol binario con cualquier tipo de type
// an ordered collectio of T's
enum BinaryTree<T> {
    Empty,
    NonEmpty(Box<TreeNode<T>>)
}

// a part of a binary Tree
struct TreeNode<T> {
    element: T,
    left: BinaryTree<T>,
    right: BinaryTree<T>
}

// Paterns:
// Recordemos la definicion de el enum RoughTime que daba mas o menos cuanto faltaba para cierto
// evento
enum RoughTime {
    InThePast(TimeUnit, u32),
    JustNow,
    InTheFuture(TimeUnit, u32)
}
// Si queremos acceder a la data que tienen los variantes del enum, debemos utilizar un match, por
// ejemplo para nuestro enum anterior:
fn rough_time_to_english(rt: RoughTime) -> String {
    match rt {
        RoughTime::InThePast(units, count)  => format!("{}{}ago", count, units.plural),
        RoughTime::JustNow                  => format!("Just Now!!!"),
        RoughTime::InTheFuture(units, count)=> format!("{}{}for now", count, units.plural)
    }
}

// Supongamos que implementamos un juego de mesa que tiene espacios hexagonales y el jugador
// clikea a donde quiere moverse
// este codigo no compila porque no podemos hacer crear
fn check_move(current_hex: Hex, click: Point) -> game::Result<Hex> {
    match point_to_hex(click) {
        None => Err("That not a game space"),
        Some(current_hex) => // try to match if user clicked the current_hex
            Err("You alredy there!!! You must click somewhere else"),
        Some(other_hex) => Ok(other_hex)
    }
}

// tuples y struct pattern: Podemos utilizar match con tuplas tambien
fn describe_point(x: i32, y: i32) -> &'static str {
    use std::cmp::Ordering::*;
    // NOTE(elsuizo:2020-04-20): aca lo que hace es comparar con el zero 0-para saber el signo
    // basicamente
    match (x.cmp(&0), y.cmp(&0)) {
        (Equal, Equal)     => "at the origin",
        (_, Equal)         => "on the x axis",
        (Equal, _)         => "on the y axis",
        (Greater, Greater) => "in the first cuadrant",
        (Less, Greater)    => "in the second quadrant",
        _                  => "somewhere else"
    }
}

// las structs-patterns usan curly braces como structs-expressions, en donde contienen un subpatron
// por cada field

match ballon.location {
    Point{x: 0, y: height}=> println!("straight up {} meters", height),
    Point{x: x, y: y}     => println!("at ({}m, {}m)", x, y)
}

// Reference patterns:
// Rust soporta dos features cuando trabajamos con referencias en un match.
// ref patterns toman "prestado" la parte que matchean
// & paterns matchean referencias
// Matcheando con valores no copiables(no implementan Copy) mueve el valor de una rama a la otra,
// por ejemplo
match account {
    Account {name, language, ..} => {
        ui.greet(&name, &language);
        ui.show_settings(&account); // error usando el valor movido `account`
    }
}
// aca los campos account.name y account.language fueron movidos a variables locales llamadas name
// y language respectivamente el resto de la struct Account fue desechada, si estas dos variables
// fueran valores "copiables" Rust copiaria los campos en lugar de moverlos y el codigo compilaria,
// pero supongamos que son Strings(que no se pueden Copy???) que se hace???
// podemos usar la palabra reservada ref
match account {
    Account {ref name, ref language, ..} => {
        ui.greet(name, language);
        ui.settings(&account); // ok!!!
    }
}
// tambien podemos usar mut ref para prestar referencias mutables
//
// El opuesto de lo anterior es el patron & que lo que hace en un match es "matchear"
// referencias
match sphere.center() {
    &Point3D{x, y, z} => ...
}
// Un patron que comienza con & va a matchear referencias en lugar de valores

// Matcheando muchaas posibilidades
//

// // TODO(elsuizo:2020-04-20): aca faltan un poco mas de los patterns con @ y los patterns que
// usan | |

//-------------------------------------------------------------------------
//                        Traits
//-------------------------------------------------------------------------
// Con esta funcionalidad se trata de lograr polimorfismo: osea que un codigo
// funcione para varios types. Rust se inspiro en Haskell typeclases. Los Traits
// de Rust son las interfaces o clases base abstractas de otros lenguajes
// Por ejemplo: el Trait Write de la libreria estandar
trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;

    // and so on...
}

// NOTE(elsuizo:2020-04-23): las funciones que implementan trait no tienen que tener como argumento
// &mut self es solo en este caso que se da

// los types de la libreria estandar que implementan Write son por ejemplo TcpStream,
// File y Vec<u8>. Todos estos types proveen los metodos llamados .write() .flush() ...
// y el codigo que utiliza el Trait Write sin importar el type se ve como:
//
use std::io::Write;

fn say_hello(out: &mut Write) -> std::io::Result<()> {
    out.write("hello world\n")?;
    out.flush()
}

// NOTE(elsuizo:2020-04-23): pero cuando ponemos que una funcion toma como argumento un parametro
// que acepta un type que implementa ese trait si tenemos que poner como &mut TraitName
// que se lee: "&mut TraitName: una referencia mutable a cualquier valor que implemente el trait
// TraitName
use std::fs::File;
let mut local_file = File::create("hello.txt")?;
say_hello(&mut local_file)?; // funciona!!!

let mut bytes = vec![];
say_hello(&mut bytes)?; // tambien funciona!!!
assert_eq!(bytes, b"hello world\n");

// Generics:
// programacion generica es otra de las variantes de polimorfismo en Rust. Como lo es en
// C++ template, una funcion generica o type puede ser usado con valores de diferentes types
//
//ejemplo:
// Dados dos valores, elegir el menor de los dos
fn min<T: Ord>(value1: T, value2: T) -> T {
    if value1 <= value2 {
        value1
    } else {
        value2
    }
}

// - Usando Traits
// la parte <T: Ord> significa que la funcion puede ser utilizada con argumentos de cualquier
// type T que implementen el Trait Ord
// Los Traits representan un capacidad que un type pude hacer, por ejemplo:
//  - Un type que implementa std::io::Write puede escribir bytes como salida
//  - Un valor que implementan std::iter::Iterator puede producir una secuencia de valores
//  - Un valor que implementa std::clone::Clone puede hacer clones de si mismo en memoria
//  - Un valor que implementa std::fmt::Debug puede ser mostrado en pantalla usando `println!()`
//  con {:?}

// Cuando queremos utilizar los metodos de un trait tenemos que importar el Trait en si explicitamente,
// por ejemplo:

// importo el Trait Write
use std::io::Write;

let mut buf: Vec<u8> = vec![];
buf.write_all(b"hello")?;

// Trait objects
// Hay dos maneras de usar traits para escribir codigo polimorfico en Rust: Trait objects y
// generics
// Rust no permite variables del type Write(un Trait) ya que el size de la variable se debe saber
// en tiempo de compilacion y los types que implementen Write pueden ser de cualquier size
// Los Traits mas utilizados son importados automaticamente por el estandar prelude
// Como no se sabe el tamanio del Trait lo que podemos hacer es una referencia a el, pero como Rust
// hace las referencias explicitas debemos cambiar el codigo.

let mut buf: Vec<u8> = vec![];
let writer: &mut Write = &mut buf;
// una referencia a un objeto Trait, como writer es conocido como trait-object, como cualquier otra
// referencia un trait-object apunta a algun valor
//
// Funciones genericas
// Vimos que podemos hacer que una funcion tome como parametro un trait como `say_hello()` podemos
// reecribir esa funcion como una funcion generica:
//
fn say_hello<W: Write>(out: &mut W) -> std::io::Result<()> {
    out.write_all(b"hello world\n")?;
    out.flush()
}

// NOTE(elsuizo:2020-04-23): aca estas diciendo que los types que le pasamos a la funcion deben
// cumplir con el Trait

// O sea que W significa para cualquier type que implemente el Trait Write
// entonces ahora podemos utlizar la funcion para cualquier type que implemente Write y utilizarla
// transparentemente:
say_hello(&mut local_file)?; // llama a say_hello::<File>
say_hello(&mut bytes)?;      // llama a say_hello::<Vec<u8>>

// Si la funcion generica no toma argumentos, es posible que tengamos que ayudar al compilador
// anotando el type que queremos, por ejemplo:
let v1 = (0..1000).collect(); // Error cant infer type
let v2 = (0..1000).collect::<Vec<i32>>(); // Ok!!!

// Cundo queremos que el type paremeter cumpla con varios Traits debemos anotarlos con un + para
// agruparlos, por ejemplo:
fn top_ten<T: Debug + Hash + Eq>(values: &Vec<T>) {
    // ...
}
// NOTE(elsuizo:2020-04-23): aca lo que le estamos diciendo es que el type que esta dentro del
// vector debe cumplir con el Trait Debug por eso es diferente al caso de la funcion `say_hello()`


// Tambien las funciones genericas pueden tener muchos parametros
// Run a query on a large, partitioned data set
// see ...
fn run_query<M: Mapper + Serialize, R: Reducer + Serialize>(data: &DataSet, map: M, reduce: R) -> Results {
    ///...
}

// cuando tenemos parametros muy largos podemos utilizar otra notacion para que quede mas prolijo,
// con la palabra reservada where
fn run_query<M,R>(data: &DataSet, map: M, reduce: R) -> Results
    where M: Mapper + Serialize,
          R: Reducer + Serialize
{
    ///...
}

// lifetime parameters van primero en la lista de parametros
// Los types alias puden ser genericos tambien
// por ejemplo:
type PancakeResult<T> = Result<T, PancakeError>;


// Cuando usar traits ???
// La eleccion de cuando usar Trait objects o codigo generico es sutil, ya que ambas
// caracteristicas estan basadas en traits, tienen muchas cosas en comun
//
// // Cuando usar traits ???
// La eleccion de cuando usar Trait objects o codigo generico es sutil, ya que ambas
// caracteristicas estan basadas en traits, tienen muchas cosas en comun.
// Traits objects son la eleccion acertada cuando queremos una coleccion de valores de types
// distintos. Por ejemplo podemos hacer una ensalada generica:
trait Vegetable {
    ///...
}

struct Salad<V: Vegetable> {
    veggies: Vec<V>
}
// Pero esto es un diseño un poco severo ya que cada nueva ensalada consiste enteramente de un solo
// type de vegetable.
// Como podemos hacer un mejor diseño???
// Dado que los valores de Vegetable puden ser todos de diferentes sizes, pero no podemos preguntar
// a Rust por un Vec<Vegetable>
//
struct Salad {
    veggies: Vec<Vegetable> // error: `Vegetable does not have a constant size`
}

struct Salad {
    veggies: Vec<Box<Vegetable>>
}

// Cada Box<Vegetable> puede tener cualquier tipo de vegetales pero el Box en si mismo tiene
// siempre un size constante(dos punteros)adecuado para guardar en un Vector. Tambien podemos
// utilizar este razonamiento para formas en una app de dibujo, personajes en un video juego,
// algoritmos de autoruteo en una red ...etc.
// Otra posible razon para usar trait objects es de reducir la cantidad de codigo compilado, ya que
// Rust puede compilar una funcion generica muchas veces una vez por cada type que es usado, esto
// hace que el binario sea largo(code bloat)
// Programacion generica tiene una importante ventaja sobre Trait objects, con el resultado de que
// en Rust programacion generica es la opcion comun, la primera ventaja es velocidad ya que el
// compilador genera codigo para cada type que es involucrado
//
//-------------------------------------------------------------------------
//                        definiendo e implementando Traits
//-------------------------------------------------------------------------
// Definir un Trait es simple, dado un nombre y lista la firma de types para los metodos del trait
// Por ejemplo si estamos escribiendo un juego:
//
// A trait of characterers, items, and scenery
// anything in the game world that's visible on screen
trait Visible {
    /// Render this object on the given canvas
    fn draw(&self, canvas: &mut Canvas);

    /// Return true if clicking at (x, y) should select this object
    fn hit_test(&self, x: i32, y: i32) -> bool;
}

// para implementar un trait, usamos: impl TraitName for Type
impl Visible for Broom {
    fn draw(&self, canvas: &mut Canvas) {
        for y in self.y - self.height - 1..self.y {
            canvas.write_at(self.x, y, '|');
        }
        canvas.write_at(self.x, self.y, 'M');
    }

    fn hit_test(&self, x: i32, y: i32) -> bool {
        self.x == x && self.y - self.height - 1 <= y && y <= self.y
    }
}

// Si queremos agregar un metodo especifico para el type que estamos implementando debemos hacerlo
// por separado, por ejemplo para el type Broom que tenemos arriba:
impl Broom {
    // Helper function used by Broom::draw() below
    fn broomstick_range(&self) -> Range<i32> {
        self.y - self.height - 1 .. self.y
    }
}

//-------------------------------------------------------------------------
//                        Default Methods
//-------------------------------------------------------------------------
// Como ejemplo podemos usar el type que hicimos antes llamado Sink, primero definimos el type:
pub struct Sink;

// Sink es una estructura vacia, dado que no necesitamos guardar ningun dato en el. Luego podemos
// proveer una implementacion de el Trait Write para Sink
//
use std::io::{Write, Result};

impl Write for Sink {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        // Clain to have sucessfully writen the whole buffer
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

// Pero como sabemos el trait Write tiene un metodo llamado `write_all()` que no lo hemos
// implementado, pero porque podemos hacer esto sin implementar ese metodo???
// porque la libreria estandar en la definicion de Write contiene una implementacion por default
// para `write_all()`

trait Write {
    // estos metodos solo se los define, por ello el que quiera implementar este trait debera
    // implementarlos
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;

    // aca se define el metodo y su implementacion por default
    write_all(&mut self, buf: &[u8]) -> Result<()> {
        let mut bytes_writen = 0;
        while bytes_writen < buf.len() {
            bytes_writen += self.write(&buf[bytes_writen..])?;
        }
        Ok(())
    }
    // ...
}

//-------------------------------------------------------------------------
//                        Traits y types de otras personas
//-------------------------------------------------------------------------

// Rust nos deja implementar cualquier trait sobre cualquier, mientras que tanto el trait como el
// type esten disponibles en el. Esto quiere decir que en cualquier tiempo que queramos añadir un
// metodo nuevo a un Type, podemos usar un trait para ello, por ejemplo:

trait IsEmoji {
    fn is_emoji(&self) -> bool
}

// Implementation IsEmoji for the built-in character type
impl IsEmoji for char {
    fn is_emoji(&self) -> bool {
        //...
        // implementacion
    }
}

assert_eq!('$'.is_emoji(), false);

// El solo proposito de este trait en particular es solo añadir un metodo a un type que ya existe,
// como `char`. Esto es llamado un trait de extension metodo a un type que ya existe, com `char`.
// Esto es llamado un trait de extensionun metodo a un type que ya existe, como `char`.
// Tambien podemos usar un imp generico para hacer toda una implementacion de toda una familia
// El siguiente trait de extension agrega un metodo a todos los Writers de Rust
use std::io::{self, Write};
// trait for values to which you can send HTML.
trait WriteHtml {
    fn write_html(&mut self, html: &HtmlDocument) -> io::Result<()>;
}

// you can write HTML to any std::io writer
impl<W: Write> WriteHtml for W {
    fn write_html(&mut self, html: &HtmlDocument) -> io::Result<()> {
        //...
    }
}

// La linea impl<W: Write> WriteHtml for W
// significa "para cualquier type W que implemente Write, aca esta la implementacion de WriteHtml
// para W"
// Un ejemplo piolisima es el de la libreria serde que nos sirve para serializar datos, osea que
// podemos escribir estructuras de datos en el disco para luego leerlas
// La libreria define un trait `Serialize` que es implementado para todos los types que la libreria
// soporta entonces en la libreria hay una implementacion de `Serialize` para los types `bool` `i8`
// `i16` ... and so on... el resultado de todo esto es que serde agrega una `.serialize()` para
// todos los types y se puede utilizar asi:

use serde::Serialize;
use serde_json;

pub fn save_configuration(config: &HashMap<String, String>) -> std::io::Result<()> {

    // create a Json serialize to write the data to a file
    let writer = File::create(config_filename())?;
    let mut serializer = serde_json::Serializer::new(writer);
    // the serde `.serialize()` method does the rest
    config.serialize(&mut serializer)?;

    Ok(())
}

//-------------------------------------------------------------------------
//                        Self en Traits
//-------------------------------------------------------------------------
// Un trait puede usar la palabra Self como type(seria como un supertype), por ejemplo:
pub trait Clone {
    fn clone(&self) -> Self
}

// Osea que nos devuelve el mismo type que el que esta llamando al metodo
//
//-------------------------------------------------------------------------
//                        SubTraits
//-------------------------------------------------------------------------
// podemos declarar que un trait es una extension de otro
//
// Someone in the game world, either the player or some other
// pixie, garoyle, squirrel, ogre, etc...
trait Creature: Visible {
    fn position(&self) -> (i32, i32);
    fn facing(&self) -> Direction;
    //...
}

// La frase `trait Creature:Visible` significa: para todas las Creature que son Visible
// Cualquier type que implemente Creature debe tambien implementar el trait Visible
impl Visible for Broom {
    //...
}
impl Creature for Broom {
    //...
}

// y podemos implementarlos en cualquier orden
//-------------------------------------------------------------------------
//                        Static Metods
//-------------------------------------------------------------------------
//
// En lenguajes orientados a objetos, las interfaces no pueden incluir metodos estaticos o
// constructores. Sin embargo, Rust Traits podemos incluir metodos estaticos y constructores, asi
// es como:
trait StringSet {
    // return a new empty set
    fn new() -> Self;
    // return a set that contains all the strings in `strings`.
    fn from_slice(strings: &[&str]) -> Self;

    // find out if this set contains a particular `value`
    fn contains(&self, string: &str) -> bool;

    // add a string to this set
    fn add(&mut self, string &str);
}

// Todos los types que quieran implementar el StringSet deberan implementar estas cuatro funciones.
// Las primeras dos no toman como argumento a self, ellas sirven como constructores.
//
// Luego en el codigo para usarlos seria algo asi:
let set1 = StringSet::new();
let set2 = StringSet::new();
// Con codigo generico podemos hacer lo mismo:
fn unknown_words<S: StringSet>(document: &Vec<String>, wordlist: &S) -> S {
    let mut unknowns = S::new();
    for word in document {
        if !wordlist.contains(word) {
            unknowns.add(word);
        }
    }
    unknowns
}

//-------------------------------------------------------------------------
//                        fully Qualified Methods calls
//-------------------------------------------------------------------------
// un metodo es una especie de funcion especial, las dos siguientes llamadas son equivalentes:
"hello".to_string();
str::to_string("hello");
<str as ToString>::to_string("hello");

// al segundo se lo llama "qualified" porque especifican el type o trait con el que el metodo esta
// hecho
// al tercero se lo llama "fully qualified" porque define no solo el type o trait sino que ademas
// de que exactamente cual es el metodo que se va a llamar
//
//-------------------------------------------------------------------------
//                  Traits que definen relaciones entre types
//-------------------------------------------------------------------------
// por ejemplo:
// - `std::iter::Iterator` relaciona cada type del iterador con el type de valores que produce
// - `std::ops::Mul` relaciona types que pueden ser multiplicados
// - `rand::Rng` incluye ambos un trait para generador de numeros random (rand::Rng) y un trait
// para types que pueden ser generados aleatoriamente (rand::Rand)
//
//-------------------------------------------------------------------------
//               Types asociados(o como funcionan los iteradores)
//-------------------------------------------------------------------------
// Como sabemos todos los lenguajes orientados a objetos tienen algun tipo de soporte para
// iteradores, objetos que representan cierta secuencia de valores.
// Rust tiene el iterador estandar definido asi:
pub trait Iterator {
    type Item;
    //...
    fn next(&mut self) -> Option<Self::Item>;
    //...
}
// Lo primero que vemos es que tiene un type asociado para el item(`type Item`). Cada type que
// implemente Iterator debe especificar que tipo de item produce
// Lo segundo es que la funcion next utiliza este type asociado en su valor de retorno
// Podemos implementar funciones genericas que contengan este type como parametro y que utilicen
// los types asociados
fn collec_into_vector<I: Iterator>(iter: I) -> Vec<I::Item> {
    let mut results = Vec::new();
    for value in iter {
        results.push(value);
    }
    results
}

// vemos que types asociados son generalmente utiles cuando un trait necesita cubrir mas que solo
// metodos
// - En un pool thread un trait `Task` representa una unidad de trabajo puede tener asociado un
// type de salida
// - Un `Pattern` trait, representando una manera de buscar un string, puede tener un type asociado
// `Match`, representando toda la informacion reunida por el patron con el string
trait Pattern {
    type Match;

    fn search(&self, string: &str) -> Option<Self::Match>;
}

impl Pattern for char {
    /// A "match" is just the location where the character was found
    type Match = usize;

    fn search(&self, string: &str) -> Option<Self::Match>;
}

// types asociados son perfectos para casos en los que cada implementacion tiene UN type
// relacionado: cada type de Task produce un type particular de Output, cada type de Pattern
// busca por un type particular de Match

/// Buddy Traits (o como funciona rand::random())
// Esta es una de las maneras mas sencillas de expresar una relacion entre types, lo que llamamos
// buddy-traits son simplemente traits que son diseniados para trabajar juntos. Hay un ejemplo muy
// conocido en el crate rand, la principal caracteristica de rand es la funcion `rand()`
use rand::random;
let x = random();
// si Rust no puede inferir el type que del `x` que queremos generar se lo tenemos que decir,
let x = random::<f64>(); // un numero entre 0.0 <= x <= 1.0
let x = random::<bool>(); // bueno un boolean

// para muchos programas esto es todo lo que necesitamos, pero este crate ofrece muchos y
// diferentes y intercambiables generadores de numeros random
// Todos los generadores de numeros aleatorios en la libreria implementan un Trait en comun

/// A random number generator
pub trait Rng {
    fn next_u32(&mut self) -> u32;
}
// osea que Rng es simplemente un valor que puede "escupir" enteros a demanda. La libreria rand
// provee algunos pocos implementaciones, incluyendo `XorShiftRng`(un pseudo generador random)
// el buddy-trait es llamado Rand:
/// A type that can be randomly generated using an `Rng`dy-trait es llamado Rand:
pub trait Rand:Sized {
    fn rand<R: Rng>(rng: &mut R) -> Self;
}
// types como `f64` y `bool` implementan este Trait. Pasando cualquier generador de numeros
// aleatorios a su metodo `::rand()` y el retorna un valor random
let x = f64::rand(rng);
let b = bool::rand(rng);
// esto es una manera elegante de separar los problemas
// Hay muchos mas casos de buddy-traits, por ejemplo: Hash ---> Hasher
// Serialize ---> Serializer

//-------------------------------------------------------------------------
//      Traits genericos(o como funciona la sobrecarga de operadores)
//-------------------------------------------------------------------------
// La multiplicacion en Rust usa este trait:
/// std::ops::Mul, the trait for types that support `*`
pub trait Mul<RHS> {
    /// the resulting type after applying the `*` operator
    type Output;

    /// the method for the `*` operator
    fn mul(self, rhs: RHS) -> Self::Output;
}

// Por ejemplo supongamos que queremos hacer una funcion que implemente el producto punto entre
// vectores o iterator
use std::ops::{Add, Mul};

fn dot<N>(v1: &[N], v2: &[N]) -> N
where N: Add<Output=N> + Mul<Output=N> + Default + Copy
{
    let mut total = N::Default();
    for i in 0..v1.len() {
        total = total + v1[i] * v2[i];
    }
    total
}

// aca lo importante es ir viendo que es lo que nos dice el compilador que nos falta cumplir para
// que compile haciendo lo que llama el autor "ingenieria inversa"
// hacemos los test para ver si funciona como pensamos que tiene que funcionar:
#[test]
fn test_dot() {
    assert_eq!(dot(&[1, 2, 3, 4], &[1, 1, 1, 1]), 10);
    assert_eq!(dot(&[53.0, 7.0], &[1.0, 5.0]), 88.0);
}
// pero investigando existe un crate muy popular llamado num que define todos los operadores y
// metodos que necesitamos para nuestra funcion!!!
use num::Num;
fn dot<N: Num + Copy>(v1: &[N], v2: &[N]) -> N {
    let mut result = N::zero();
    for i in 0..v1.len() {
        result = result + v1[i] * v2[i];
    }
    result
}
// como en OP la interface correcta hace que todo sea bonito, en programacion generica un trait
// adecuado hace que todo se vea mas lindo.
// La ventaja de esto comparado con templates de C++ es que con traits estamos poniendo limites
// donde el compilador puede mirar compatibilidades hacia "arriba". Podemos cambiar la
// implementacion de cualquier metodo generico publico y si no cambiamos la firma de la misma el
// usuario no se enterara. Otra ventaja sobre templates de C++ son los errores que puede dar el
// compilador ya que este puede decirnos en que linea y en que archivo esta el problema, ademas
// cuando vemos una funcion generica podemos inferir cuales son los parametros que recibe y como se
// comporta en cambio con templates no...
//
//
//-------------------------------------------------------------------------
//                  Cap:12 Sobrecarga de operadores
//-------------------------------------------------------------------------
// podemos hacer que nuestros types soporten operaciones aritmeticas basicas, implementando algunos
// traits esto se llama sobrecarga de operadores
// los traits para sobrecarga de operadores terminan en unas pocas categorias, dependiendo sobre
// que parte del lenguaje soporta(ver tabla 12-1).
// En Rust la operacion `a + b` es en realidad `a.add(b)` o sea que a llama al metodo `add` de la
// libreria estandar `std::ops::Add`
// Hay mas maneras de sobrecargar operadores
// Cuando en un type parameter ponen `where Rhs: ?Sized` esto hace que Rust relaje su requerimiento
// de que deben ser types con size como Strings, Vecs o HashMaps
//
// La siguiente tabla es un resumen de los traits que implementan operadores en los que se puede
// sobrecargar su uso con nuestros types
----------------------+-----------------------+-------------------------------+
Category              | Trait                 |  Operator                     |
----------------------+-----------------------+-------------------------------+
Unary Operators       | std::ops::Neg         |  -x                           |
                      | std::ops::Not         |  !x                           |
----------------------+-----------------------+-------------------------------+
Aritmetic Operators   | std::ops::Add         |  x + y                        |
                      | std::ops::Sub         |  x - y                        |
                      | std::ops::Mul         |  x * y                        |
                      | std::ops::Div         |  x / y                        |
                      | std::ops::Rem         |  x % y                        |
----------------------+-----------------------+-------------------------------+
Bitwise Operators     | std::ops::BitAnd      |  x & y                        |
                      | std::ops::BitOr       |  x | y                        |
                      | std::ops::BitXor      |  x ^ y                        |
                      | std::ops::Shl         |  x << y                       |
                      | std::ops::Shr         |  x >> y                       |
----------------------+-----------------------+-------------------------------+
Compound assigment    | std::ops::AddAssign   |  x += y                       |
arithmetic operators  | std::ops::SubAssign   |  x -= y                       |
                      | std::ops::MulAssign   |  x *= y                       |
                      | std::ops::DivAssign   |  x /= y                       |
                      | std::ops::RemAssign   |  x %= y                       |
----------------------+-----------------------+-------------------------------+
Compound assigment    | std::ops::BitAndAssign|  x &= y                       |
bitwise operators     | std::ops::BitOrAssign |  x |= y                       |
                      | std::ops::BitXorAssign|  x ^= y                       |
                      | std::ops::ShlAssign   |  x << y                       |
                      | std::ops::ShrAssign   |  x >> y                       |
----------------------+-----------------------+-------------------------------+
Comparison            | sts::cmp::PartialEq   |  x == y, x != y               |
                      | std::cmp::PartialOrd  |  x < y, x <= y, x > y, x >= y |
----------------------+-----------------------+-------------------------------+
Indexing              | std::ops::Index       |  x[y], &x[y]                  |
                      | std::ops::IndexMut    |  x[y] = z, &mut x[y]          |
----------------------+-----------------------+-------------------------------+
//-------------------------------------------------------------------------
//                        Arithmetic and Bitwise Operators
//-------------------------------------------------------------------------
// Como sabemos en Rust `a + b` es una abreviacion de `a.add(b)` una llamada a el metodo `add()`
// del Trait `std::ops::Add`, recordemos que si queremos hacer por ejemplo: `z.add(c)` debemos
// tener en el scope el trait, osea:
use std::ops::Add;
// y la definicion del trait es la siguiente:
trait Add<RHS=Self> {
    type Output;
    fn add(self, rhs: RHS) -> Self::Output;
}

// Por ejemplo si queremos sumar dos numeros complejos podemos hacer:
// NOTE: como estamos sumando dos numeros complejos del mismo type, no necesitamos decir cual es el
// type de RHS en Add<RHS>
impl Add for Complex<i32> {
    type Output = Complex<i32>;
    fn add(self, rhs: Self) -> Self::Output {
        Complex{re: self.re + rhs.re, im: self.im + rhs.im}
    }
}

// pero como sabemos podemos implementar toda la familia de types numericos en una sola definicion
// con generics:
impl<T> Add for Complex<T>
    where T: Add<Output=T>
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Complex{re: self.re + rhs.re, im: self.im + rhs.im}
    }
}
// escribiendo `where T: Add<Output=T>` estamos restringiendo al type T que puede sumarse con el
// mismo. Pero el trait Add no requiere que los dos operandos sean del mismo type. Entonces una
// implementacion mas generica seria que los dos operandos varien independientemente y que
// produzcan un valor `Complex<T>` como resultado
impl<L, R, O> Add<Complex<R>> for Complex<L>
where L: Add<R, Output=O>
{
    type Output = Complex<O>;
    fn add(self, rhs: Complex<R>) -> Self::Output {
        Complex{re: self.re + rhs.re, im: self.im + rhs.im}
    }
}

// en la practica sin embargo Rust trata de evitar soportar operaciones de types mezclados. Dado
// que nuestro parametro `L` debe implementar `Add<R, Output=O>` esto generalmente sigue que `L`,
// `R` y `O` todos terminan siendo el mismo type ya que simplemente no hay tantos types disponibles
// que cumplan la restriccion

//-------------------------------------------------------------------------
//                        Ordered Comparison
//-------------------------------------------------------------------------
// Rust especifica el comportamiento de operadores de comparamiento <, >, <=, >= todo en terminos
// de un simple trait, el std::cmp::PartialEq
trait PartialOrd<Rhs=Self>: PartialEq<Rhs> where Rhs: ?Sized {
    fn partial_cmp(&self, other: &Rhs) -> Option<Ordering>;

    fn lt(&self, other: &Rhs) -> bool
    fn le(&self, other: &Rhs) -> bool
    fn gt(&self, other: &Rhs) -> bool
    fn ge(&self, other: &Rhs) -> bool
}
// recordemos que el ?Sized relaja el type parameter para que pueda ser un type que no posee un
// Size definido(por ejemplo un puntero).
// Notamos que `PartialOrd` extiende `PartialEq` o sea que podemos hacer comparaciones de
// ordenacion solo para los types en los cuales tambien podemos comparar por igualdad(de eso se
// trata los subtraits no ???)
// El unico metodo de `PartialOrd` que tenemos que implementar es `partial_cmp()`. Cuando esta
// funcion retorna `Some(o)` entonces `o` indica la relacion que tiene con el otro type:
enum Ordering {
    Less,    // self < other
    Equal,   // self == other
    Greater  // self > other
}
// Si sabemos que los valores de dos types estan siempre ordenados uno con respecto al otro,
// entonces lo que podemos implementar es el trait mas estricto: `std::cmp::Ord`
trait Ord: Eq + PartialOrd<Self> {
    fn cmp(&self, other: &Self) -> Ordering;
}

// como vemos el metodo `cmp()` solo retorna un `Ordering` en lugar de un `Option<Ordering>` ya que
// `cmp()` siempre declara sus argumentos como iguales (ya que como vemos tiene que impl `Eq`) o
// indica su orden relativo (o sea no puede haber un `None`)
// Como no podemos comparar numeros complejos vamos a utilizar el siguiente ejemplo: Sea el type
// que representan un conjunto de numeros que estan en un cierto intervalo semi-abierto:
#[derive(Debug, PartialEq)]
struct Interval<T> {
    lower: T, // inclusive
    upper: T, // exclusive
}
// deseariamos hacer que los valores de este type sean ordenados parcialmente: un intervalo es
// menor que otro si este cae enteramente antes del otro, sin solapamientos. Si dos intervalos que
// no son iguales se solapan seran desordenados: algunos elementos de cada lado seran menores que
// algunos elementos del otro lado y dos intervalos que son iguales son simplemente iguales(ja). La
// siguiente implementacion de `PartialOrd` implementa estas reglas:
use std::cmp::{Ordering, PartialOrd};

impl<T: PartialOrd> for PartialOrd<Interval<T>> for Interval<T> {
    fn partial_cmp(&self, other: &Interval<T>) -> Option<Ordering> {
        if self == other{Some(Ordering::Equa)}
        else if self.lower >= other.upper {Some(Ordering::Greater)}
        else if self.upper <= other.lower {Some(Ordering::Less)}
        else {None}
    }
}
// con esta implementacion podemos escribir:
assert!(Interval{lower: 10, upper: 20} < Interval{lower: 20, upper: 40});
assert!(Interval{lower: 7, upper: 8} >= Interval{lower: 0, upper: 1});
assert!(Interval{lower: 7, upper: 8} <= Interval{lower: 7, upper: 8});

// y como sabemos los Intervalos que se solapan no los podemos ordenar, osea:
let left  = Interval{lower: 10, upper: 30};
let right = Interval{lower: 20, upper: 40};
assert!(!(left < right));
assert!(!(left >= right));


//-------------------------------------------------------------------------
//                        Index y IndexMut
//-------------------------------------------------------------------------
// podemos especificar como sera el comportamiento de una expresion de indexacion como a[i] en
// nuestros types, implementando los traits std::ops::Index y std::ops::IndexMut. Los arrays
// soportan el operador [] directamente pero sobre cualquier otro type, la expresion a[i] es una
// abreviacion para `*a.index(i)` donde index es un metodo del trait std::ops::Index, donde estas
// son las definiciones:

trait Index<Idx> {
    type Output: ?Sized;
    fn index(&self, index: Idx) -> &Self::Output;
}

trait IndexMut<Idx>: Index<Idx> {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output;
}

// El uso mas comun para indexar es con colecciones. Por ejemplo, supongamos que tenemos una
// imagen en bitmap, como la del ejemplo de Mandelbrot, recordemos que nuestro programa tenia
// codigo como este:
pixels[row * bounds.0 + column] = //...
// estaria bueno tener un type para las imagenes como: `Image<u8>` que actue como un arrays de dos
// dimensiones de pixels, permitiendonos acceder a los pixels sin tener que hacer toda la
// aritmetica, osea:
image[row][column] = //...

// Por ello primero definimos el type
struct Image<P> {
    width: usize,
    pixels: Vec<P>
}

impl<P: Default + Copy> Image<P> {
    /// Create a new image of the given size
    fn new(width: usize, height: usize) -> Image<P> {
        Image{
            width,
            pixels: vec![P::default(); width * height]
        }
    }
}
// y la implementacion de Index y de IndexMut son las siguientes:
//
impl<P> std::ops::Index<usize> for Image<P> {
    type Output = [P];
    fn index(&self, row: usize) -> &[P] {
        let start = row * self.width;

        &self.pixels[start..start + self.width] // lo que devuelve son referencias a los pixels
    }
}

impl<P> std::ops::IndexMut<usize> for Image<P> {
    fn index_mut(&mut self, row: usize) -> &mut [P] {
        let start = row * self.width;
        &mut self.pixels[start..start + self.width]
    }
}

//-------------------------------------------------------------------------
//                  Cap: 13 Utility Traits
//-------------------------------------------------------------------------
// Aparte de la sobrecarga de operadores que vimos existen mas traits que pertenecen a la libreria
// std que podemos implementar para nuestros types:
//  - Podemos usar el trait Drop cuando queremos especificar cuando y como limpiar los valores de
//  una variable cuando salen de un "scope" como los destructores de C++
//  - Los punteros "inteligentes" como `Box<T>` y `Rc<T>` pueden implementar el trait `Deref` para
//  hacer que el puntero refleje los metodos del valor "envuelto" (wrapped)
//  - Si implementamos `From<T>` y `Into<T>` podemos decirle a Rust como queremos que convierta un
//  valor a otro
//
// Drop: Cuando el duenio de un valor se pierde decimos que Rust "drops" el valor. "Dropping" un
// valor implica liberar cualquiera de los otros valores a los cuales el duenio tenia acceso,
// espacio en el heap y recursos del sistema que el valor ocupaba.
//
// Sized: Un type se dice que es `Sized` si cualquiera de sus valores tienen el mismo size en
// memoria. Casi todos los types de Rust son `Sized`. Aunque un `Vec<T>` tiene su propia region de
// memoria en el heap allocada cuyo size puede variar, el valor de `Vec` en si mismo es un puntero
// a el buffer, a su capacidad y su tamanio, entonces `Vec<T>` es un type `Sized`
// NOTE: no me lo esperaba a esto eh
// Sin embargo Rust tiene unos pocos types que no tienen el mismo `Sized`. Por ejemplo el type
// "str slice" (notemos que es sin el puntero!!!) no tiene un sized definido. Los literales de
// strings "diminutivo" y "big" son referencias a "str slice" que ocupan 10 y 3 bytes. Los dos que
// son "Array slice" como `[T]` (nuevamente sin el puntero & !!!) no tienen size tambien una
// referencia compartida como `&[u8]` puede apuntar a un slice `[u8]` de cualquier tamanio. Porque
// los types `str` y `[T]` denotan un conjunto de valores de tamanios variables, ellos son types
// que no tienen size!!!
// El otro type que no tiene `Sized` son los "trait-objects", que como sabemos un "trait-object" es
// un puntero a algun valor que implementa un trait dado. Por ejemplo, los types: `&std::io::Write`
// y `Box<std::io::Write>` son punteros a algun valor que implementa ese trait y como no se sabe de
// antemano(en tiempo de compilacion) cual es el type que nos van a pasar, por eso se dice que no
// tienen `Sized`. En Rust no podemos guardar en variables o pasarlas como argumento a valores que
// no tienen `Sized`, solo podemos utilizarlas via punteros como `&str` o `Box<Write>` los cuales
// si tienen size (porque son punteros basicamente)
// Todos los `Sized` implementan el trait `std::marker::Sized` que no tiene metodos asociados solo
// se usa como un "marcador". Rust lo implementa automaticamente para nuestros types que apliquen,
// por eso no tenemos que implementarlo nosotros mismos. El unico uso para `Sized` es como limite
// como: `T: Sized` que hace que `T` sea un type cuyo size sea conocido en tiempo de compilacion,
// estos traits se conocen como "markers traits" ya que Rust los utiliza para "marcar" ciertas
// caracteristicas de interes en los types.
// Ya que los types que no tienen size son tan limitados la mayoria de los types genericos deben
// restringirse a que sean `Sized`, de hecho es necesario tan a menudo que Rust lo toma como un
// default implicito, ya que si escribimos `struct S<T>{...}` lo que Rust interpreta es `struct
// S<T: Sized> {...}`. Si no queremos que tenga esa restriccion implicita lo que tenemos que
// escribir explicitamente es `struct S<T: ?Sized> {...}`. Donde `?Sized` quiere decir: "no
// necesariamente sized". Por ejemplo si escribimos `struct S<T: ?Sized> {b: Box<T>}` entonces Rust
// nos permitira escribir `S<str>` o `S<Write>`.
// Cuando el type de una variable tiene la restriccion `?Sized` la gente dice que tiene un sized
// cuestionable: o sea que puede ser `Sized` o no
//
// Clone: El trait `std::clone::Clone` es para types que pueden hacer copias de si mismos. Clone se
// define de la siguiente manera:
trait Clone: Sized {
    fn clone(&self) -> Self;

    fn clone_mut(&mut self, source: &Self) {
        *self = source.clone()
    }
}
// O sea que el metodo `clone()` debe construir una copia independiente de si misma y retornarla.
// Dado que este metodo retorna un type como `Self` y las funciones pueden no retornar valores sin
// size, el trait `Clone` extiende a el trait `Sized` esto tienen como efecto de restringir a las
// implementaciones del trait a que el el propio type sea `Sized`(o sea que clone solo se puede
// implementar para types que tengan size). Recordemos que clonar algo conlleva a allocar memoria
// para esa copia, entonces clonar puede ser costoso, en ambos tiempo y memoria. Por ejemplo clonar
// un `Vec<String>` no solo copia el vector sino que tambien copia cada uno de los `String` que
// estan en el, por eso Rust no clona los valores automaticamente en cambio hace que tengamos que
// llamar a un metodo explicito. Los punteros del tipo con contador de referencia como `Rc<T>` y
// `Arc<T>` son excepciones clonar uno de estos simplemente incrementa el contador de referencia y
// nos da una nueva referencia
//
// Copy: En el cap 4, explicamos que para muchos types la asignacion mueve el valor en lugar de
// copiarlos
let str1 = "piolisima".to_string();
let str2 = str1;

let num1: i32 = 37;
let num2      = num1;

// lo que vimos que hace es que `str1` y `str2` mueven el valor a la misma direccion de memoria
// en cambio cuando asignamos una variable `i32` la copia.
// Ahora podemos definir que es un type `Copy`: es el que implementa el trait `std::marker::Copy`
// que se define:
trait Copy: Clone {}
// que hace que sea muy facil de implementar para nuestros types:
impl Copy for MyType {}

// Deref y MutDeref: Podemos especificar como queremos que los operadores como `*` y `.` se comporten
// para nuestros types implementando los traits `std::ops::Deref` y `std::ops::DerefMut`. Los types
// del tipo puntero como `Rc<T>` y `Box<T>` implementan estos traits entonces se pueden comportar
// como punteros de Rust nativos, por ejemplo si tenemos un valor b `Box<Complex>` entonces `*b` se
// refiere al valor `Complex` al que apunta `b` y `b.re` se refiere a su componente real. Si el
// contexto asigna o toma prestada una referencia mutable al referente entonces Rust utiliza el
// trait `DerefMut` ("Derederencia Mutable") de otra manera acceso de solo lectura es suficiente y
// usa `Deref`. Los traits son definidos asi:
trait Deref {
    type Target: ?Sized;
    fn deref(&self) -> &Self::Target;
}

trait DerefMut: Deref {
    fn dered_mut(&mut self) -> &mut Self::Target;
}
// tanto `deref()` como `deref_mut()` toman una `&Self` y retornan `&Self::Target`. `Target` debe
// ser algo que `Self` contiene, posee o se refiere, por ejemplo con `Box<Complex>` el type es
// `Complex` notemos que `DerefMut` extiende a `Deref` ya que si podemos desreferenciar algo y
// modificarlo ciertamente podremos tomar prestado una referencia a ello tambien. Dado que los
// metodos retornan una referencia con los mismos lifetimes que `&self` entonces `self` permanece
// prestado tanto tiempo como la referencia retornada viva.
// `Deref` y `DerefMut` juegan otro papel importante dado que `deref()` toma una `&Self` y retorna
// una `&Self::Target` Rust usa esto para convertir automaticamente referencias del primer type en
// el segundo. En otras palabras insertando una llamada a `deref` puede prevenir que dos types no
// coincidan e implementando `DerefMut` nos habilita la correspondiente conversion para referencias
// mutables. Estas son llamadas "deref coercions": un type es coaccionado a comportarse como otro.
// Aunque Rust las hace por nosotros las podemos escribir explicitamente nosotros, ellas son
// convenientes:
// - Si por ejemplo tenemos una variable `r = Rc<String>` y queremos aplicar `String::find('?')` a
// ella para encontrar ese `char` en `r` podemos escribir simplemente: `r.find('?')` en lugar de
// hacer la desreferencia explicita osea: `(*r).find('?')` : el metodo llama implicitamente a a una
// referencia prestada de `r` y `&Rc<String>` es coaccionado a comportarse como `&String` porque
// `Rc<T>` implementa `Deref<Target=T>`
// - Podemos usar metodos como `split_at` sobre valores del type `String` aun cuando `split_at()`
// es un metodo de "str slice" pero ya que `String` implementa `Deref<Target=str>`. Por ello no es
// necesario para `String` reimplementar todos los metodos de `str` dado que podemos coaccionar a
// `&str` desde &String
// - Si tenemos un vector de bytes `v` y queremos pasarselo a una funcion que espera un slice de
// bytes `&[u8]` podemos simplemente pasarle `&v` como argumento dado que `Vec<T>` implementa
// `Deref<Target=[T]>`
// Rust aplica varios coerciones "deref" si es necesario. Por ejemplo usando las coerciones
// mencionadas antes podemos aplicar `split_at` directamente a un `Rc<String>` dado que
// `&Rc<String>` desreferencia a `&String` que a su vez dereferencia a `&str` el cual tiene el
// metodo `split_at`. Por ejemplo supongamos que tenemos el siguiente type:
struct Selector<T> {
    /// elements avaible in this Selector
    elements: Vec<T>,

    /// The index of the current element in `elements`. A `Selector` behaves like a poiter to the
    /// current element
    current: usize
}
// Para hacer que el type se comporte como dice el doc del mismo tenemos que implementar `Deref` y
// `DerefMut`:
use std::ops::{Deref, DerefMut};
impl<T> Deref for Selector<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.elements[self.current]
    }
}

impl<T> DerefMut for Selector<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.elements[self.current]
    }
}
// Dadas estas implementaciones podemos usar a `Selector` como:
let mut s = Selector{elements: vec!['x', 'y', 'z'], current: 2};
// ya que `Selector` implementa `Deref` y `DerefMut` podemos utilizar el operador `*` para
// desreferenciar y obtener el elemento actual (`current`)
assert!(*s, 'z');
// podemos utilizar un metodo de 'char' directamente sobre `Selector` via coercion
assert!(s.is_alphabetic());
// podemos cambiar 'z' por 'w' asignando la desreferencia de `Selector`
*s = 'w';
assert_eq!(s.elements, ['x', 'y', 'w']);
// NOTE: esto es locooo eh
// Los traits `Deref` y `DerefMut` estan diseniados para implementar types a la "smart pointers"
// como `Rc`, `Arc`y `Box` y types que sirven como versiones propietarias de algo que usamos
// tambien frecuentemente como una referencia la manera de que `Vec<T>` y `String` sirven como
// versiones propietarias de `[T]` y `str`
//
// Default: Algunos types tiene un razonable valor obvio por default, el default para un vector es
// un vector nulo, para un numero el zero, el de `Option` es `None` y asi...Types asi pueden
// implementar el trait `std::default::Default`:
trait Default {
    fn default() -> Self;
}
// este metodo `default()` simplemente devuelve un valor fresco del type `Self`, por ejemplo la
// implementacion de `Default` para `String` es sencilla:
impl Default for String {
    fn default() -> Self {
        String::new()
    }
}
// Todos los types que son `collections` de cosas implementan el `Default` como la colleccion
// vacia, esto es util cuando necesitamos construir una coleccion de valores pero queremos dejarle
// a la funcion que llamamos que decida cual es la mejor coleccion que puede construir, por ejemplo
// el metodo del trait `Iterator` `partition()` "parte" en dos a nuestro iterador y produce dos
// colecciones nuevas, usando un closure para decidir a donde va cada valor:
use std::collections::HasSet;
let squares = vec![4, 9, 16, 25, 36, 49, 64];
let (power_of_two, impures): (HashSet<i32>, HashSet<i32>) = squares.iter().partition(|&n| n & (n - 1) == 0);
assert_eq!(power_of_two.len(), 3);
assert_eq!(impures.len(), 4);
// NOTE: el clousure loco ese verifica si es par o no pero a nivel bits :O
// Otro uso que se le da al este trait es cuando queremos que un type tenga valores de
// inicializacion de algo y son muchos entonces los ponemos todos en un type que implemente
// `Default`
// Si un type T implementa `Default` entonces la libreria standar implementa automaticamente
// `Default` para `Rc<T>`, `Arc<T>`, `Box<T>`, `Cell<T>`, `RefCell<T>`, `Cow<T>`, `Mutex<T>` y
// `RwLock<T>`. El valor de `Default` para el type `Rc<T>` por ejemplo es un `Rc` que apunta a el
// valor por default para el type T
// Rust no implementa implicitamente default para los `struct`, pero si todos los campos de la
// `struct` implementan `Default` podemos implementar Default para la `struct` automaticamente
// usando el viejo truco de `#[derive(Default)]`
//
// AsRef y AsMut: Cuando un type implementa `AsRef<T>` quiere decir que podemos pedir prestado una
// &T de el eficientemente. `AsMut` es el analogo para referencias mutables. Las definiciones son
// las siguientes:
trait AsRef<T: ?Sized> {
    fn as_ref(&self) -> &T;
}

trait AsMut<T: ?Sized> {
    fn as_mut(&mut self) -> &mut T;
}
// por ejemplo `Vec<T>` implementa `AsRef<[T]>` y `String` implementa `AsRef<str>`. Podemos tambien
// pedir prestado una referencia de una `String` como un array de bytes ya que `String` implementa
// tambien `AsRef<[u8]>`
// `AsRef` es usada tipicamente para hacer que las funciones sean mas flexibles en los argumentos
// que aceptan. Por ejemplo la funcion std `std::fs::File::open` es declarada asi:
fn open<P: AsRef<Path>>(path: P) -> Result<File>
// lo que `open()` necesita es realmente un `&Path`, pero con esta declaracion la funcion acepta
// cualquier argumento que pueda pedir prestado un `&Path` (o sea cualquiera que implemente
// `AsRef<Path>`) Algunos de estos types incluyen `String` `str` y mas...Por esto es que podemos
// pasarle un literal a la funcion `open()`:
let dot_emacs = std::fs::File::open("/home/elsuizo/.emacs");
//
// Borrow and BorrowMut: El trait `std::borrow::Borrow` es similar a `AsRef`: si el type implementa
// `Borrow<T>` entonces su metodo de prestamo toma eficientemente un `&T` pero este trait impone mas
// restricciones: un type debe implementar `Borrow<T>` solo cuando un `&T` "hashes" y se compare de
// la misma manera como el valor es prestado(Rust no obliga esto; es solo el espiritu documentado en
// el trait). Esto hace que `Borrow<>` sea valioso cuando trabajamos con "keys" y tablas "hash" o
// cuando trabajamos con valores que pueden ser "hashed" o comparados por alguna otra razon
// Esta distincion cuenta cuando pedimos prestado de una `String` por ejemplo: `String` implementa
// `AsRef<&str>`, `AsRef<[u8]>` y `AsRef<Path>` pero estos tres target van a generar diferentes
// "hashes". Solo el `&str` se garantiza que "hashea" como el equivalente `String`, entonces
// `String` solo implementa `Borrow<str>`. La definicion de `Borrow<>` es identica a `AsRef<>`;
// solo que cambia el nombre:
trait Borrow<Borrowed: ?Sized> {
    fn borrow(&self) -> &Borrowed;
}
// NOTE(elsuizo:2020-05-08): aca sigue con algunas cositas de como seria implementar un type que
// requiere algun tipo de Hash
//
// From y Into: Los traits `std::convert::From` y `std::convert::Into` representan conversiones que
// consumen un valor de un type y retornan el valor de otro. Donde los traits `AsRef` y `AsMut`
// toman prestado una refencia de uno de los types hacia el otro, `From` y `Into` toman propiedad
// de sus argumentos, los transforman y retornan la propiedad del resultado de vuelta al que llamo
// la funcion. Sus definiciones son simetricas como sigue:
trait Into<T>: Sized {
    fn into(self) -> T;
}
trait From<T>: Sized {
    fn from(T) -> Self;
}

// la libreria standar implementa automaticamente la conversion trivial desde cada type a si mismo:
// todos los types T implementan `From<T>` y `Into<T>`
// Aunque los traits simplemente proveen dos maneras de hacer la misma cosa ellos se prestan para
// usos distintos. Genenralmente usamos `Into` para hacer nuestras funciones mas flexibles en los
// argumentos que acepta. Por ejemplo, si escribimos:
use std::net::Ipv4Addr;
fn ping<A>(address: A) -> std::io::Result<bool>
where A: Into<Ipv4addr>
{
    let ipv4_address = address.into();
}
// Entonces el metodo `ping()` puede aceptar no solo un `Ipv4Addr` como argumento sino que tambien
// un `u32` o un `[u8; 4]` dado que estos types ambos implementan `Into<Ipv4Addr>`. Como vimos
// cuando tratabamos `Paths` con `AsRef` el efecto es como el de sobrecargar a la funcion hablando
// en terminos de C++. Entonces con esta definicion de `ping()` podemos hacer las siguientes
// llamadas:
println!("{:?}", ping(Ipv4Addr::new(23, 21, 68, 141))); // le pasamos un Ipv4Addr puro
println!("{:?}", ping([66, 146, 219, 98]);              // le pasamos un [u8; 4]
println!("{:?}", ping(0xd076eb94_u32);                  // le pasamos un u32
//
// El `From` trait en cambio cumple un rol diferente. El metodo `from()` sirve como un constructor
// generico para producir una instancia de un type desde otro valor. Por ejemplo en lugar de
// `Ipv4Addr` tenga dos metodos llamados `from_array` y `from_u32` simplemente implementamos
// `From<[u8; 4]>` y `From<u32>` permitiendonos escribir:
let addr1 = Ipv4Addr::from([66, 146, 219, 98]);
let addr2 = Ipv4Addr::from(0xd076eb94_u32);

// Asi podemos dejar que la inferencia de types haga su trabajo.
// Dado una apropiada implementacion de `From` la libreria std automaticamente implementa el
// correspondiente `Into`. Cuando definimos nuestros propios types, si tienen una sola linea de
// argumentos su constructor tendremos entonces que escribir una implementacion de `From<T>` para
// los apropiados types entonces obtendremos "gratis" las correspondientes implementaciones de
// `Into`!!!
// Ya que los metodos de conversion `into()` y `from()` toman propiedad de sus argumentos una
// conversion puede reusar los recursos de el valor original para construir el nuevo valor. Por
// ejemplo:
let text = "repiolisima".to_string();
let bytes: Vec<u8> = text.into();
// La implementacion de `Into<Vec<u8>>` de `String` simplemente toma el buffer de el `String`
// allocado en el heap y lo reutiliza sin alterar para formar el `Vec<u8>` que retorna. La
// conversion no necesita allocar o copiar el text!!!
// Este es otro caso donde "moves" permite una implementacion eficiente
//
// ToOwned: Dada una referencia, la manera usual de producir una copia propia de lo que apunta mi
// referencia es llamar al metodo `clone()`, asumiendo que el type implemeta el trait
// `std::clone::Clone` pero que pasa si queremos "clonar" un `&str` o un `&[i32]`??? lo que
// probablemente queremos es un `String` o un `Vec<i32>` pero la definicion de `clone()` no permite
// esto: Por definicion clonar un `&T` debe siempre retornar un valor del type `T` y `str` y [u8]
// no tienen sized!!! no son siquiera types que una funcion pueda retornar???
// el trait `std::borrow::ToOwned` provee una manera menos restrictiva de convertir una referencia
// a un valor propio:
trait ToOwned {
    type Owned: Borrow<Self>;
    fn to_owned(&self) -> Self::Owned;
}
// diferente a `clone()` el cual retorna el type `T` o `Self` `to_owned` retorna el type que
// nosotros le decimos cuando implementamos el trait
//
// Borrow y ToOwned trabajando:(el caso del humilde `Cow`)
// Hacer un buen uso de Rust involucra muchas veces en pensar como tratar el tema de la propiedad
// de los datos, como una funcion recibira parametros; por referencia o por valor???. Usualmente
// podemos adoptar una o otra y los parametros(types) de las funciones reflejaran esta descicion.
// Pero en algunos casos no podemos decidir cuando pedir prestado o tener la propiedad hasta que el
// programa este corriendo(o sea no en tiempo de compilacion), el type `std::borrow::Cow` ("clone
// on write") provee una manera de hacer esto, su definicion es esta:
enum Cow<'a, B: ?Sized + 'a>
where B: ToOwned
{
    Borrowed(&'a B),
    Owned(<B as ToOwned>::Owned),
}
// A `Cow<B>` o toma prestada una referencia compartida a B o posee un valor del cual podriamos
// tomar prestada dicha referencia. Dado que `Cow` implementa `Deref` podemos llamar metodos sobre
// el como si fueran una referencia compartida a `B`: si es de su propiedad toma prestado una
// referencia compartida a el valor que es de nuestra propiedad y si es prestado solo reparte la
// referencia que tiene.
// Podemos tambien obtener una referencia mutable a un valor `Cow` llamando su metodo `to_mut` el
// cual retorna una `&mut B`. Si sucede que que es `Cow::Borrowed` entonces `to_mut()` simplemente
// llama a el metodo de la referencia `to_owned()` para obtener su propia copia de la referencia,
// cambia el `Cow` a `Cow::Owned` y toma prestada una referencia mutable a el nuevo valor recien
// adquirido. Por esta razon es que se llama "clone on write".
// Similarmente, `Cow` tiene un metodo llamado `into_owned()` que promueve la referencia a un valor
// adquirido si es necesario y entonces lo retorna, moviendo la propiedad a la funcion que lo llama
// y consumiendo el `Cow` en el proceso.
// Un uso comun de `Cow` es retornar o un string constante statico o un string que es producto de
// una funcion en tiempo de ejecucion. Por ejemplo, supongamos que tenenmos un mensaje de error
// dentro de un `enum`. Muchas de las variantes se pueden tratar con strings fijas, pero algunas
// que requieren ser formateadas en relacion al contexto en que son llamadas. Para ello podemos
// retornar un `Cow<'static, str>`:
use std::path::PathBuf;
use std::borrow::Cow;

fn describe(error: &Error) -> Cow<'static, str> {
    match *error {
        Error::OutOfMemory   => "out of memory".into(),
        Error::StackOverflow => "stack overflow".into(),
        Error::MachineOnFire => "machine on fire".into(),
        Error::Unfathomable  => "machine bewildered".into(),
        // aca es cuando pasamos un string que depende del contexto
        Error::FileNotFound(ref path) => {
            format!("file not found: {}", path.display()).into()
        }
    }
}
// Este codigo utiliza la implementacion de `into()` para construir los valores. La mayoria de los
// brazos del `match` retornan un `Cow::Borrowed` refiriendose a las string allocadas staticamente,
// pero cuando llega al brazo `FileNotFound` como usamos un `format!` para constuir el mensaje
// incorporando el nombre del file, este brazo produce un `Cow::Owned`. Los que llaman a la
// funcion `describe()` no necesitan cambiar el valor pueden simplemente tratar el `Cow` como un
// `&str`:
println!("Disaster has struck: {}", describe(&error));
// los que llaman a la funcion que necesitan adquirir la propiedad de un valor pueden facilmente
// producir uno:
let mut log: Vec<String> = Vec::new();
// ...
log.push(describe(&error).into_owned());
// Usando `Cow` ayudamos a la funcion `describe()` y los que la llaman ponen la allocacion de
// memoria hasta el momento que sea necesario.
//
//
//-------------------------------------------------------------------------
//                        14: Closures
//-------------------------------------------------------------------------
// Ordenar un vector de enteros es facil:
integers.sort();

// Casi siempre cuando queremos ordenar algo no es una lista de enteros tipicamente tenemos una
// lista con custom types:
struct City {
    name: String,
    population: i64,
    country: String,
    ...
}

// Si hacemos ahora:
fn sort_cities(cities: &mut Vec<City>) {
    cities.sort(); // ---> error como queremos ordenarlas???
}
// Rust se queja de que tenemos que implementar `std::cmp::Ord` debemos especificar el criterio que
// debe utilizar para ordenarlos
/// Helper function for sorting cities by population
fn city_population_descending(city: &City) -> i64 {
    -city.population
}

fn sort_cities(cities: &mut Vec<City>) {
    cities.sort_by_key(city_population_descending); // Ok!!!
}

// Esta funcion `city_population_descending()` toma una `City` y extrae la key que es el criterio
// que queremos para ordenar a las ciudades (esta retorna un numero negativo porque sort ordena a
// los numeros en forma creciente y lo que queremos es en forma decreciente). La funcion
// `sort_by_key()` toma esta funcion como parametro. Esto funciona bien pero es mas conciso si
// creamos una funcion anonima directamente en el parametro!!!
fn sort_cities(cities: &Vec<City>) {
    cities.sort_by_key(|city| -city.population);
}

// aca lo que creamos es lo que se conoce como "closure" (una funcion anonima, creo que por eso
// viene lo de salve al mundo cree una funcion anonima ya que ahorra todo el proceso de crear la
// funcion y luego llamarla)
//
// Capturando variables:
// Los closures pueden usar variables que le pertenecen a la funcion que esta llamando al closure,
// por ejemplo:
/// sort by any different statistics
fn sort_by_statistics(cities: &mut Vec<City>, stat: Statistic) {
    cities.sort_by(|city| -city.get_statistic(stat))
}

// Esta utilizando la variable `stat` dentro del closure!!!
//
// Closures que piden prestado:
// del ejemplo anterior la funcion: `sort_by_statistics()` cuando Rust crea el closure, este
// automaticamente pide prestado una referencia a `stat` es logico: el closure se refiere a stat,
// entonces debe tener una referencia a el. Y de esta manera el closure sigue las reglas que vimos
// de lifetimes y prestamos de referencias, en particular como el closure contiene una referencia a
// `stat`, Rust no no dejara que esta referencia viva mas alla del scope que tiene el closure. Por
// eso Rust asegura que es seguro a traves del lifetime y no necesita un GC.
//
// Closures que roban: El segundo ejemplo es un poco mas complicado:
use std::thread;

fn start_sorting_thread(mut cities: Vec<City>, stat: Statistic) -> thread::JoinHandle<Vec<City>> {
    let key_fn = |city: &City| -> i64 {-city.get_statistics(stat)};

    thread::spawn(|| {
        cities.sort_by_key(key_fn);
        cities
    })
}

// Esto es mas o menos como lo hace JavaScript `thread::spawn` toma un closure y lo llama en un
// nuevo "thread" (notemos que `||` significan que no recibe argumentos) el nuevo "thread" corre en
// paralelo con la funcion que lo llama. Cuando el closure retorna el nuevo "thread" desaparece(el
// valor de retorno del closure es enviado a el "thread" que lo llamo como un valor de type: `JoinHandle`
// (esto se ve en el capitulo 19)
// Nuevamente el closure `key_fn` pero en esta vez no puede garantizar que la referencia vaya a ser
// utilizada de manera correcta, por ello la funcion anterior no compila. De hecho hay dos
// problemas aqui porque `cities` esta siendo compartida de manera insegura tambien. Es muy simple
// el nuevo "thread" creado por `thread::spawn` no puede esperar finalizar su trabajo antes de que
// `cities` y `stat` sean destruidas(porque las esta utilizando el!!!). La solucion a los dos
// problemas es la misma: Rust nos dice que "movamos" a `cities` y `stat` a el closure que los usa
// en lugar de prestarle una referencia a el.
//
fn start_sorting_thread(mut cities: Vec<City>, stat: Statistic) -> thread::JoinHandle<Vec<City>> {
    let key_fn = move |city: &City| -> i64 {-city.get_statistics(stat)};

    thread::spawn(move || {
        cities.sort_by_key(key_fn);
        cities
    })
}
// como vemos la unica cosa que cambio fue que pusimos la palabra reservada `move` antes de cada
// uno de los closures. Este palabra reservada hace que el closure no pida prestada una referencia
// de la variable sino que la robe
//
// Funciones y types de closures: Como vimos hasta ahora usamos los closures como valores, por ello
// deben tener un type asignado. Por ejemplo:
fn city_population_descending(city: &City) -> i64 {
    -city.population
}

// esta funcion toma un argumento del type `&City` y retorna un `i64`. Tiene entonces el type
// `fn(&City)->i64`. Podemos hacer todo lo mismo que hacemos con los otros type, por ejemplo los
// `struct` pueden tener campos genericos con `Vec` que en su interior tengan todas el mismo type
// de funcion, en memoria las funciones son un simple puntero que apunta al codigo de maquina de la
// funcion, como un puntero a funcion en C. Una funcion puede tomar otra funcion como argumento:
/// Given a list of cities and a test function,
/// return how many cities pass the test
fn count_selected_cities(cities: &Vec<City>, test_fn: fn(&City)-> bool) -> usize {
    let mut count = 0;
    for city in cities {
        if test_fn(city) {
            count += 1;
        }
    }
    count
}

/// An example of a test function. Note that the type of this function is `fn(&City)->bool`, the
/// same as the `test_fn` argument to `count_selected_cities`
fn has_moster_attacks(city: &City) -> bool {
    city.monster_attack_risk > 0.0
}

let n = count_selected_cities(&my_cities, has_moster_attacks);

// Los closures tienen distintos types que las funciones vistas como variables, por ejemplo en el
// ejemplo anterior tendriamos que cambiar el type de la funcion que se recibe:
fn count_selected_cities<F>(cities: &Vec<City>, test_fn: F) -> usize
where F: Fn(&City) -> bool
{
    let mut count = 0;
    for city in cities {
        if test_fn(city) {
            count += 1;
        }
    }
    count
}

// Vemos que la nueva version es generica para todas las funciones que implementen el trait
// especial `Fn(&City)->bool`. Este trait es automaticamente implementado por todas las funciones
// y closures que toman como argumento un `&City` y retornan un `bool`
// fn(&City) -> bool ---> fn type para funciones
// Fn(&City) -> bool ---> Fn trait para ambas funciones y closures
//
// Entonces la nueva version generica de `count_selected_cities()` acepta tanto closures como
// funciones normales
//
// Closure Performance: Los closures son diseniados para ser rapidos mas o tan rapidos como
// punteros a funciones, lo suficientemente rapidos para que pueden ser usados en codigo que
// requiere mucha performance. Los closures de Rust no tienen los problemas de performance que
// tienen en los lenguajes con GC(por ejemplo Julia) no son allocados en el Heap a menos que los
// pongamos dentro de un `Box` un `Vec` o otro contenedor y como se sabe el type de antemano Rust
// puede hacer optimizaciones en linea con ellos
//
// Closures y seguridad:
// Que pasa cuando un closure deja o modifica una variable capturada
// Closures que Matan: hasta ahora vimos que closures que piden prestado valores y closure que
// roban valores. En la terminologia de Rust no se matan a los valores sino que se los "drop", la
// manera mas trivial de hacer esto es llamar a la funcion `drop()`:
let my_string = "hello".to_string();
let f = || drop(my_string);
// cuando llamamos al clousure `my_string` es "drop", pero entonces que pasa si llamamos de nuevo a
// esta funcion???
// f(); // ok
// f(); // error: use of moved value
// O sea que Rust save que este clousure no se puede llamar mas de una vez!!!
//
// FnOnce: Veamos un truco mas para este ejemplo, esta vez vamos a utilizar este funcion generica:
fn call_twice<F>(clousure: F)
    where F: Fn()
{
    closure();
    closure();
}
// Esta funcion generica le podemos pasar cualquier closure que implemente el trait `Fn()` o sea un
// closure que no tiene argumentos y que retorna `()` como com las funciones el type que retorna
// `()` se puede obviar
let my_string = "hello".to_string();
let f = || drop(my_string);
call_twice(f);

// Nuevamente Rust se da cuenta que queremos hacer algo que no podemos(hacer un doble free!!!)
// Rust le da un type especial a los closures que "drop" values pero no le da el type `Fn`. Estos
// closures impl un trait con menos libertades `FnOnce` el trait para closures que solo pueden ser
// llamados una sola vez
//
// FnMut: Existe otro tipo de closure, el que contiene datos mutables o referencias mutables.
// Rust considera valores no mutables seguros para ser compartidos a traves de threads, pero seria
// seguro compartir closures no mutables que contienen data mutable, llamar a un closure asi podria
// debenir en "race-conditions" (muchos threads tratando de leer y escribir al mismo tiempo). Por
// ello Rust tiene una categoria mas de closure `FnMut` estos closures son llamados por una
// referencia mutable. Cualquier closure que requiera un acceso mutable a un valor pero no "drop"
// ningun valor es un `FnMut`, por ejemplo:
let mut i = 0;
let incr = || {
    i += 1; // incr borrows a mut reference to i
    println!("Ding i is now: {}", i);
};
call_twice(incr);

// Recapitulando un poco de las tres categorias de closures:
//
//  - `Fn`: Es una familia de closures y funciones que podemos llamar muchas veces sin
//  restricciones. Esta categoria mas alta tambien incluye a el tipe de funciones comunes `fn`
//  - `FnMut`: Es una familia de closures que se pueden llamar muchas veces si el closure en si
//  mismo es declarado como `mut`
//  - `FnOnce`: Es la familia de closures que pueden ser llamados una sola vez, si el que lo llama
//  tiene como propiedad al propio closure
//
// O sea que `Fn` es un subtrait de `FnMut` que a su vez es un subtrait de `FnOnce`, esto hace que
// `Fn` sea la mas poderosa y exclusiva categoria de closures
//
//
// Callbacks: Muchas librerias utilizan callbacks como parte de su API: funciones que son provistas
// por el usuario, para que la libreria las utilice o las llame luego, usamos una API asi en el
// cap2 cuando utilizamos el framework iron para escribir un simple web server que lucia mas o
// menos asi:
fn main() {
    let mut router = Router::new();
    router.get("/", get_from, "root");
    router.post("/gcd", post_gcd, "gcd");

    println!("Serving on http://localhost:3000...");
    iron::new(router).http("localhost:3000").unwrap();
}

// el proposito del router es routear los mensajes que vienen de internet, en este ejemplo los
// closures son `get_from` y `post_gcd` que son los nombres de funciones que declaramos en algun
// lugar del codigo
//
// Supongamos que queremos escribir nuestro propio simple router:
// podemos declarar unos types para representar un request "HTTP" y sus respuestas

struct Request {
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Vec<u8>
}

struct Response {
    code: u32,
    headers: HashMap<String, String>,
    body: Vec<u8>
}

// Ahora el trabajo de el router es simplemente guardar una tabla que mapea URLs a callbacks,
// entonces el callback que necesitamos lo podemos llamar si lo requerimos(por simplificacion del
// problema solo permitimos que los usuarios creen routes que matcheen exactamente con el URL)
struct BasicRouter<C>
where C: Fn(&Request) -> Response
{
    routes: HashMap<String, C>
}

impl<C> BasicRouter<C>
where C: Fn(&Request) -> Response
{
    // create a empty Router
    fn new() -> BasicRouter<C> {
        BasicRouter{routes: HashMap::new()}
    }

    fn add_route(&mut self, url: &str, callback: C) {
        self.routes.insert(url.to_string(), callback);
    }
}

// desafortunadamente tenemos un error. Este Router funciona bien hasta que solo agregamos un
// solo router a el. Nuestro error fue en como definimos `BasicRouter` en el que definimos que el
// callback type sea uno solo y todos los callbacks que vamos poniendo en el `HashMap` son del
// mismo type!!!, pasa lo mismo  cuando queriamos hacer el type `Salad`. La solucion como lo fue
// aquella vez tambien es que tenemos que hacer un Box de ellos
type BoxedCallback = Box<Fn(&Request) -> Response>;

struct BasicRouter {
    routes: HashMap<String, BoxedCallback>
}

// Esto requiere unos ajustes en los metodos
impl BasicRouter {
    // create a basic empty router
    fn new() -> Self {
        BasicRouter {routes: HashMap::new()}
    }

    fn add_route<C>(&mut self, url: &str, callback: C)
        where C: Fn(&Request) -> Response + 'static
    {
        self.routes.insert(url.to_string(), Box::new(callback));
    }

}
// ahora nuestro router esta listo para recibir los mensajes!!!:
impl BasicRouter {
    fn handle_request(&self, request: &Request) -> Response {
        match self.routes.get(&request.url) {
            None => not_found_response(),
            Some(callback) => callback(request)
        }
    }
}

// Usando closures efectivamente
// Hay que pensar un poco mas que en los otros lenguajes a la hora de utilizar closures ya que no
// tenemos un GC que nos "ayude". Por ejemplo tomemos el modelo super conocido como MVC (Model View
// Controller). Por cada elemento en una GUI MVC crea tres objetos: un modelo(que representa el
// estado del elemento en la GUI) una vista(que es resposable de como se ve el elemento) y un
// controlador que maneja la interaccion con el usuario. Tipicamente cada objeto tiene una
// referencia del otro objeto o de ambos, directamente o con un callback, cualquier cosa que pase
// con uno es notificado cualquiera de los otros. La pregunta es que objeto posse a los demas
// nunca aparece. Por eso no podemos implementar este patron en Rust sin hacer algunos cambios ya
// que la propiedad debe hacerse explicita y los ciclos de referencia deben eliminarse. El modelo y
// el controlador no pueden tener referencias directas unos a otros. Se pueden solucionar estos
// problemas de diferentes maneras o podemos utilizar un modelo de flujo de datos que sea
// unidireccional como la arquitectura de Flux en Facebook.
//
//-------------------------------------------------------------------------
//                        cap: 15 Iterators
//-------------------------------------------------------------------------
//
// Un iterator es un valor que produce una secuencia de valores, tipicamente para operar en un loop
// Consideremos la siguiente funcion:
fn triangle(n: i32) -> i32 {
    let mut sum = 0;
    for i in 1..n+1 {
        sum += 1;
    }
    sum
}

// aqui la expresion 1..n+1 es un valor `Range<i32>` que es un iterador que produce enteros desde
// su valor inicial hasta su valor final
// Pero Itertors tiene un metodo `fold` el cual puede usarse para una definicion equivalente que la
// anterior:
fn triangle2(n: i32) -> i32 {
    (1..n+1).fold(0, |sum, item| sum + item)
}

// Los traits `Iterator` y `IntoIterator`:
//
// Un iterador es cualquier valor que implemente el trait `std::iter::Iterator`
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    //...
    //...
    // muchos metodos default mas
}
// aqui `Item` es el type que produce el iterador, el metodo `next` retorna un `Option<Self::Item>`
// o sea que produce un `None` o un `Some(Item)`. Si existe una manera "natural" de iterar sobre
// algun type, entonces podemos impl el trait `std::iter::IntoIterator` cuyo metodo `into_iter()`
// toma un valor y retorna un iterador sobre el.
trait IntoIterator
where Self::IntoIter::Item == Self::Item {
    type Item;
    type IntoIter: Iterator;

    fn into_iter(self) -> Self::IntoIter;
}

// `IntoIter` es el type del iterador y `Item` es el type del valor que produce. Llamamos a
// cualquier type que implementa `IntoIterator` como iterable, porque es algo al cual podemos
// iterar sobre el. Como sabemos un vector es iterable ya que si escribimos el siguiente `for` loop
// lo podemos comprobar:
println!("There's");
let v = vec!["antimony", "arsenic", "aluminum", "selenium"];
for element in &v {
    println!("{}", element);
}

// Bajo el capot todos los `for` loops son una notacion abreviada para llamar a los metodos
// `IntoIterator` e `Iterator`:
let mut iterator = (&v).into_iter();
while let Some(element) = iterator.next() {
    println!("{}", element);
}
// Todos los `Iterators` implementan automaticamente `IntoIterator` con un metodo `into_iter` que
// simplemente retorna un iterador.
// Algunas terminologias para iterators:
//  - Como dijimos un iterator es cualquier type que implemente `Iterator`
//  - Un iterable es cualquier type que implemente `IntoIterator`: podemos obtener un iterator
//  sobre el llamando al metodo `into_iter()`. La referencia al vector del ejemplo `&v` es el
//  iterable en este caso
//  - Un iterador produce valores
//  - Los valores que un iterador produce son `items`. Aqui los items son "antimony" ...
//  - El codigo que recibe el item que un iterador produce es el consumer. En este ejemplo el loop
//  `for` consume los items del iterador
//
// Creando Iteradores:
// Los metodos `iter` y `iter_mut`:
// Muchos colecciones de types proveen los metodos `iter` y `iter_mut` que retornan iteradores
// naturales sobre el type, produciendo una referencia compartida o una referencia mutable a cada
// item. Slices como `&[T]` y `&str` tienen un `iter` y un `iter_mut` tambien
// Cada type es libre de implementar `iter` y `iter_mut` de la manera que quiera. Por ejemplo el
// metodo `iter` sobre `std::path::Path` retorna un iterador que produce un componente del path por
// vez:
use std::ffi::OsStr;
use std::path::Path;

let path = Path::new("C:/Users/JimB/Downloads/Fedora.iso");
let mut iterator = path.iter();

assert_eq!(iterator.next(), Some(OsStr::new("C:")));
assert_eq!(iterator.next(), Some(OsStr::new("Users")));
assert_eq!(iterator.next(), Some(OsStr::new("JimB")));

// implementaciones de `IntoIterator`:
// Cuando un type implementa `IntoIterator` podemos llamar al metodo `into_iter` como lo haria un
// loop `for`
// deberiamos usar `HashSet` pero su orden de iteracion no es deterministico, entonces `BtreeSet`
// funciona mejor para los propositos de este ejemplo:
use std::collections::BTreeSet;
let mut favorites = BTreeSet::new();
favorites.insert("Lucy in the Sky with Diamonds".to_string());
favorites.insert("liebestramu Nº3".to_string());

let mut it = favorites.into_iter();
assert_eq!(it.next(), Some("liebestramu Nº3".to_string())));
assert_eq!(it.next(), Some("Lucy in the Sky with Diamonds".to_string()));
assert_eq!(it.next(), None); // no hay mas elementos!!!

// Podemos utilizar al trait, podemos utilizar una restriccion como: `T: IntoIterator` para
// restringir a los types que solo puedan iterar sobre el.
// Por ejemplo esta funcion imprime los valores de cualquier iterable cuyos items puedan ser
// mostrados con el formato; `"{:?}"` podemos escribir `T: IntoIterator<Item=U>` para requerir que
// el type que devuelva sea de un determinado type `U`. Por ejemplo la siguiente funcion imprime
// los valores que provienen de cualquier iterable cuyos items son printables
use std::fmt::Debug;

fn dump<T, U>(t: T)
    where T: IntoIterator<Item=U>,
          U: Debug
{
    for u in t {
        println!("{:?}", u);
    }
}

// Metodos que "Drain":
// Muchas colecciones proveen un metodo `drain` que toma una referencia mutable a la coleccion y
// retornan un iterador que pasa la propiedad de cada elemento al consumidor. Sin embargo diferente
// a al metodo `into_iter()` el cual toma a la coleccion por valor y la consume, `drain`
// simplemente pide prestado una referencia mutable a la coleccion y cuando el iterador es "tirado"
// este metodo remueve cualquier elemento que quede en la coleccion y la deja limpia. Sobre types
// que pueden ser indexados por un `range` como `String` `Vec` `VecDeques` el metodo `drain` toma
// un rango de elementos para ser removidos en lugar de vaciar toda la sequencia
//
use std::iter::FromIterator;

let mut outer = "Earth".to_string();
let inner = String::from_iter(outer.drain(1..4));
assert_eq!(outer, "Eh");
assert_eq!(inner, "art");

// Otras fuentes de iteradores
//
// Hay muchas fuentes de iteradores en la libreria std
//
------------------------------+-------------------------------------+-------------------------------------------------------------------------------------+
Type o trait                  |   Expresion                         |      Notas                                                                          |
------------------------------+-------------------------------------+-------------------------------------------------------------------------------------+
std::ops::Range               | 1..10                               |  el final debe ser un int                                                           |
------------------------------+-------------------------------------+-------------------------------------------------------------------------------------+
std::ops::RangeFrom           | 1..                                 |  Iteracion sin final. El valor inicial debe ser un int.                             |
                              |                                     |  Puede `panic` o averflow si el valor llega al limite del type                      |
------------------------------+-------------------------------------+-------------------------------------------------------------------------------------+
Option<T>                     | Some(10).iter()                     |  Se comporta como un vector cuyo length es o cero(None) o 1(Some(v))                |
------------------------------+-------------------------------------+-------------------------------------------------------------------------------------+
Result<T, E>                  | Ok("blah").iter()                   |  Similar a Option, produciendo valores Ok                                           |
------------------------------+-------------------------------------+-------------------------------------------------------------------------------------+
                              | v.windows(16)                       |  Produce un slice continuo del length que le pasamos.                               |
                              +-------------------------------------+-------------------------------------------------------------------------------------+
                              | v.chuncks(16)                       |  Produce valores contiguos que no se solapan, dado el length de                     |
                              |                                     |  izquierda a derecha                                                                |
 Vec<T>, &[T]                 +-------------------------------------+-------------------------------------------------------------------------------------+
                              | v.chuncks_mut(1024)                 |  Como el anterior pero produce slices mutables                                      |
                              +-------------------------------------+-------------------------------------------------------------------------------------+
                              | v.split(|byte| byte & 1 != 0)       |  Produce slices separados por elementos que "matchean" el predicado dado            |
                              +-------------------------------------+-------------------------------------------------------------------------------------+
                              | v.slit_mut(..)                      |  Como el anterior pero produce slices que son mutables                              |
                              +-------------------------------------+-------------------------------------------------------------------------------------+
                              | v.rsplit(..)                        |  Como `split()` pero produce slices de derecha a izquierda                          |
                              +-------------------------------------+-------------------------------------------------------------------------------------+
                              | v.splitn(n, ...)                    |  Como `split()` pero produce a lo sumo `n` slices                                   |
------------------------------+-------------------------------------+-------------------------------------------------------------------------------------+
                              | s.bytes()                           |  Produce los bytes de la forma UTF-8                                                |
                              +-------------------------------------+-------------------------------------------------------------------------------------+
                              | s.chars()                           |  Produce los chars que representan a los UTF-8                                      |
                              +-------------------------------------+-------------------------------------------------------------------------------------+
                              | s.split_whitespace()                |  Splits un string donde haya espacios en blanco y produce un slice sin              |
                              |                                     |  espacios en blanco                                                                 |
                              +-------------------------------------+-------------------------------------------------------------------------------------+
                              | s.lines()                           |  Produce un slice que contiene las lineas del slice(por ej: un txt)                 |
                              +-------------------------------------+-------------------------------------------------------------------------------------+
 String, &str                 | s.split('/')                        |  Splits un dado string sobre un patron dado produciendo un slice que                |
                              |                                     |  consta de los elementos que no matchearon. Los patrones pueden ser muchas          |
                              |                                     |  cosas: chars, strings, clousures                                                   |
                              +-------------------------------------+-------------------------------------------------------------------------------------+
                              | s.matches(char::is_numeric)         |  Produce slices que matchean un patron dado (en este caso que sean numeros)         |
------------------------------+-------------------------------------+-------------------------------------------------------------------------------------+
                              | set1.union(set2)                    |  Produce referencias copartidas a los elementos de la union de los set1 y set2      |
 std::collections::HashSet    +-------------------------------------+-------------------------------------------------------------------------------------+
 std::collections::BTreeSet   |                                     |  Produce referencias compartidas a los elementos de la interseccion de              |
                              | set1.intersection(set2)             |  los set1 y set2                                                                    |
------------------------------+-------------------------------------+-------------------------------------------------------------------------------------+
 std::sync::mpsc::Receiver    | recv.iter()                         |  Produce valores enviados por otro thread sobre el correspondiente Sender           |
------------------------------+-------------------------------------+-------------------------------------------------------------------------------------+
                              | stream.bytes()                      |  Produces bytes from an I/O stream                                                  |
 std::io::Read                +-------------------------------------+-------------------------------------------------------------------------------------+
                              | stream.chars()                      |  Parsea streams como UTF-8 y produce chars                                          |
------------------------------+-------------------------------------+-------------------------------------------------------------------------------------+
                              | bufstreams.lines()                  |  Parsea streams como UTF-8 produce lineas como Strings                              |
 std::io::BufRead             +-------------------------------------+-------------------------------------------------------------------------------------+
                              | bufstreams.split(0)                 |  Split stream sobre un dado byte produce un buffer inter-byte Vec<u8>               |
------------------------------+-------------------------------------+-------------------------------------------------------------------------------------+
 std::fs::ReadDir             | std::fs::read_dir(path)             |  Produce una lista de los directorios que hay en un cierto path                     |
------------------------------+-------------------------------------+-------------------------------------------------------------------------------------+
 std::net::TcpListener        | listener.incoming(path)             |  Produce una lista de las conexiones que tenemos                                    |
------------------------------+-------------------------------------+-------------------------------------------------------------------------------------+
                              | std::iter::empty()                  |  Retorna None inmediatamente                                                        |
    Funciones libres          +-------------------------------------+-------------------------------------------------------------------------------------+
                              | std::iter::once(5)                  |  Produce el valor dado y luego termina                                              |
    Funciones libres          +-------------------------------------+-------------------------------------------------------------------------------------+
                              | std::iter::repeat("#7")             |  Produce el valor dado para siempre                                                 |
------------------------------+-------------------------------------+-------------------------------------------------------------------------------------+


// Adaptadores de iteradores
// Una vez que tenemos un iterador en la mano el trait `Iterator` provee una amplia seleccion de
// metodos para adaptar, que consumen el iterador para generar uno nuevo, con comportamientos
// utiles. Para ver como los "adaptadores" trabajan vamos a ver como trabajan dos de los mas
// populares:
//
// `map` y `filter`:
// El adaptador `map` nos permite modificar los elementos de un iterador aplicando un closure a sus
// items. este adaptador nos permite "filtrar" items desde un iterador, usando un closure para
// decidir cual mantener y cual eliminar. Por ejemplo supongamos que queremos iterar sobre las
// lineas de un texto y queremos eliminar los espacios en blanco de cada linea. El metodo de la
// libreria estandar `std::trim` elimina los espacios en blanco que hay al comienzo y al final de
// las lineas de un `&str` retornando un nuevo `&str` que toma prestado de la original. Podemoso
// entonces usar el adaptador `map` para aplicar `std::trim` a cada linea:
//
// NOTE(elsuizo:2020-05-11): esa no la sabia que podemos poner un `map` con el nombre de la
// funcion solo y que llame a la funcion directamente
let text = "ponies \n girafas \n iguanas \n squid".to_string();
let v: Vec<&str> = text.lines().map(str::trim).collect();
assert_eq!(v, ["ponies", "girafas", "iguanas", "squid"]);

// por ejemplo si quisieramos excluir a "ponies" de nuesto vector podriamos hacerlo asi:
let v: Vec<&str> = text.lines().map(str::trim).filter(|s| *s != "ponies").collect();
assert_eq!(v, ["girafas", "iguanas", "squid"]);

// Aqui filter retorna un tercer iterator que produce items solo cuando la condicion del clousure
// es `true`
//
// Dos cosas importantes cosas sobre adaptadores de iteradores:
// 1. Llamando un adaptador sobre un iterador no consume ningun item, solo retorna un nuevo
//    iterador
// 2. Los adaptadores de iteradores no tienen "overhead" por realizar esta abstraccion. Ya que como
//    vimos `map` y `filter` son genericos aplicandose ellos sobre un iterador que tiene items de
//    un determinado type. Esto quiere decir que Rust tiene la suficiente informacion para hacer
//    que cada metodo `next` sea "inline"
//
// `filter_map` y `flat_map`: El adaptador `map` esta bien en situaciones donde cada item que
// recibe produce uno de salida. Pero que pasa si queremos borrar ciertos items en lugar de
// procesarlos, o reemplazar items con alguno o ningun item???. `filter_map` y `flat_map` nos dan
// esta flexibilidad.
// El adaptador `filter_map` es similar a `map` exepto que deja que el clousure cree un nuevo
// elemento (como lo hace `map`) o que lo tire("drop") de la iteracion. Asi es como una combinacion
// de `filter` y `map`, su signatura es la siguiente:
fn filter_map<B, F>(self, f: F) -> some Iterator<Item=B>
where Self: Sized, F: FnMut(Self::Item) -> Option<b>;

// Es la misma signatura que `map` exepto que aqui el closure retorna un `Option<B>` no simplemente
// `B`. Cuando el closure retorna `None` el item es desechado de la iteracion; cuando retorna
// `Some(b)` entonces `b` es item que `filter_map` produce. Por ejemplo supongamos que queremos
// scanear un string que tiene palabras separadas por espacios en blanco tal que podamos parsearlas
// como numeros, procesar estos numeros y descartar las otras palabras o letras
use std::str::FromStr;

let txt = "1\nfrond .25 289\n3.1415 estuary\n";
for number in txt.split_whitespace().filter_map(|w| f64::from_str(w).ok()) {
    println!("{:4.2}", number.sqrt());
}

// Este closure dado por `filter_map` intenta parsear cada palabra que esta separada por un
// espacio en blanco usando `f64::from()` que retorna un `Result<f64, ParseFloatError>` el cual lo
// podemos convertir en un `Option<f64>` con el metodo `ok()` (para que `filter_map` pueda
// utilizarlo) mientras que un error de parseo se convierte en un `None` y un parseo exitoso se
// convierte en un `Some(v)`. Entonces `filter_map` descarta a todos los `None` y produce el valor
// `v` para cada `Some(v)`. Podriamos hacer lo mismo con un `map` y un `filter` pero no queda muy
// elegante:
txt.split_whitespace().map(|w| f64::from_str(w)).filter(|r| r.is_ok()).map(|r| r.unwrap())

// Podemos pensar a el adaptador `flat_map` como una continuacion en el mismo sentido de `map` y
// `filter_map` exepto que ahora el closure puede retornar no solo un item(como con `map`) o zero o
// un item (como con `filter_map`) pero si una sequencia de cualquier numero de items(provenientes
// de un solo item). El adaptador `flat_map` produce una concatenacion de sequencias que produce un
// closure. La signatura de `flat_map` es la siguiente:
fn flat_map<U, F>(self, f: F) -> some Iterator<Item=U::Item>
where F: FnMut(Self::Item) -> U, U: IntoIterator;

// El closure que le pasamos a `flat_map` debe retornar un iterable. Por ejemplo supongamos que
// tenemos una tabla que mapea Paises con sus ciudades mas importantes. Dada una lista de paises
// como podemos iterar sobre sus ciudades mas importantes
use std::collections::HashMap;

let mut major_cities = HashMap::new();
major_cities.insert("Japan", vec!["Tokio", "Kyoto"]);
major.cities.insert("The United States", vec!["Portland", "Nashville"]);
major.cities.insert("Brazil", vec!["Sao Paulo", "Brasilia"]);
major.cities.insert("Kenya", vec!["Nairobi", "Mombasa"]);
major.cities.insert("The netherlands", vec!["Amsterdam", "Utretch"]);

let countries = ["Japan", "Brazil", "Kenya"];

for &city in countries.iter().flat_map(|country| &major_cities[country]) {
    println!("{:}", city);
}

// Una manera de ver esto es: por cada pais sacamos el vector de ciudades lo concatenamos todos los
// vectores en una sola secuencia y los imprimimos
//
// Pero recordemos que los iteradores son "lazy" o vagos ya que solo utilizan a el item cuando
// tiene que utilizar un valor y el vector de todas las ciudades concatenados no se construye nunca
// en memoria
//
// `scan`: Este adaptador se asemeja a `map` exepto que el closure que le pasamos es dado como un
// valor mutable que el puede consultar y tiene la opcion de terminar la iteracion antes. Toma un
// valor de estado inicial y un closure que acepta una referencia mutable a el estado y el proximo
// item de el iterador subyacente. El closure debe retornar un `Option` el cual el iterador `scan`
// toma como su proximo item.
// Por ejemplo veamos una cadena de iteradores que eleban al cuadrado otro valor de un item de un
// iterador y termina la iteracion una vez que su suma exede 10
//
let iter = (0..10).scan(0, |sum, item| {
    *sum += item;
    if *sum > 10 {
        None
    } else {
        Some(item * item)
    }
});

assert_eq!(iter.collect::Vec<i32>(), vec![0, 1, 4, 9, 16]);


// `take` y `take_while`: estos adaptadores nos dejan terminar una iteracion despues de un cierto
// numero de items o cuando un closure decide terminar la iteracion. Sus signaturas son las
// siguientes:
fn take(self, n: usize) -> some Iterator<Item=Self::Item>
where Self:Sized;

fn take_while<P>(self, predicate: P) -> some Iterator<Item=Self::Item>
where Self: Sized, P: FnMut(&Self::Item) -> bool;

// Ambos toman propiedad de un iterador y retornan un nuevo iterador que es pasado a lo largo de
// los items desde el primero, posiblemente terminando la secuencia mas temprano. El iterador
// `take` retorna un `None` despues de producir a lo sumo `n` items. El iterador `take_while`
// aplica el predicado a cada item y retorna `None` en lugar donde los items producen un `false` al
// predicado del clousure y para cada subsecuente llama a el proximo. Por ejemplo, dado un mail con
// una linea blanca separando el header del body del mensaje, podemos utilizar `take_while` para
// iterar solo en los headers:
//
// NOTE(elsuizo:2020-05-12): no me queda muy claro porque le pasamos `!line.is_empty()` en lugar de
// `line.is_empty()`
let message = "To: jimb\r\n
                From: superego<editor@oreily.com>\r\n
                \r\n
                Did you get any writing done today???\r\n
                When will you stop wasting time plotting fractals???\r\n";

for header in meassage.lines().take_while(|line| !line.is_empty()) {
    println!("{:}", header);
}

// Recordemos de "String Literals" que cuando una linea en un string finaliza con un "\" Rust no
// incluye la indentacion de la proxima linea en el string, entonces ninguna de estas lineas tiene
// ningun espacio en blanco al principio. Esto significa que la tercer linea del mensaje que
// pusimos como ejemplo esta en blanco. El adaptador `take_while()` termina la iteracion ni bien el
// ve esa linea blanca(que en el codigo se representa con `!line.is_empty()`)
//
// `skip` y `skip_while`: estos iteradores son el complemento de los anteriores: Ellos eliminan un
// cierto numero de items desde el comienzo de una iteracion o eliminan items hasta que un closure
// encuentra uno aceptable y entonces pasa los items que quedan sin cambio, las signaturas son como
// siguen:
fn skip(self, n: usize) -> some Iterator<Item=Self::Item>
where Self: Sized;

fn skip_while<P>(self, predicate: P) -> some Iterator<Item=Self::Item>
where Self: Sized, P: FnMut(&Self::Item) -> bool

// uno de los usos comunes de `skip` es saltear el nombre del comando cuando estamos iterando sobre
// un programa que recibe comandos de la linea de comandos, nuestra calculadora de maximo comun
// denominador usa el siguiente codigo:
for arg in std::env::args().skip(1) {

}

// la funcion `std::env::args()` retorna un iterador que produce los argumentos que le pasamos al
// programa como una string, el primer item siempre es el nombre del programa en si mismo que no
// nos sirve en este caso y por eso lo salteamos
// `skip_while` usa un closure para decidir cuantos items eliminar desde el comienzo de la
// secuencia. Podemos hacer el equivalente del ejemplo anterior donde iterabamos sobre los headers
// de un mail pero en este caso iterando sobre el body:
for body in message.line().skip_while(|line| !line.is_empty()).skip(1) {
    println!("body: {:}", body);
}

// `peekeable`: Un iterador es nos deja pegar una ojeada al item que vamos a producir sin
// consumirlo. Podemos convertir a casi todos los iteradores en un iterador peekeable llamando al
// metodo del trait `peekeable()`:
fn peekeable(self) -> std::iter::Peekeable<Self>
where Self: Sized;

// Aqui `Peekeable<Self>` es una estructura que implementa `Iterator<Item=Self::Item>` y `Self` es
// el type del iterador subyacente. Un iterador peekeable tiene un metodo adicional que retorna un
// `Option<&Item>`: `None` si el iterador subyacente ha terminado y de otra manera `Some(r)`, donde
// `r` es la referencia compartida al proximo item(notemos que si el type del iterador es ya una
// referencia a algo esto termina siendo una referencia a una referencia)
// Estos iteradores son esenciales cuando no podemos decidir de antemano cuantos items consumir de
// un iterador hasta que hemos ido ya lejos. Por ejemplo si queremos parsear numeros de un stream
// de caracteres no podemos decidir cuando el numero finaliza hasta que no hemos visto el proximo
// char que no sea un numero:
use std::iter::Peekeable;

fn parse_number<I>(tokens: &mut Peekeable<I>) -> u32
where I: Iterator<Item=char>
{
    let mut n = 0;
    loop {
        match tokens.peek() {
            Some(r) if r.is_digit(10) => {
                n = n * 10 + r.to_digit(10).unwrap();
            },
            _ => return n
        }
        tokens.next();
    }
}

// La funcion `parse_number` usa `peek` para chequear el proximo caracter y consume a este solo si
// es un digito o el iterador ha concluido(que es cuando peek retorna `None`) hacemos que retorne
// cuando hemos parseado un numero exitosamente dejando el proximo character en el iterador, listo
// para ser consumido
//
// `fuse`: Una vez que un iterador ha retornado `None` el trait no especifica como deberia
// comportarse si llamamos al metodo nuevamente, muchos iteradores solo retornan `None` de vuelta
// pero no todos. El adaptador `fuse` toma cualquier iterator y lo convierte en uno que puede
// continuar retornando `None` una vez que hemos concluido con la iteracion(o sea hemos llegado al
// primer `None`)
struct Flaky(bool);

impl Iterator for Flaky {
    type Item = &'static str;
    if next(&mut self) -> Option<Self::Item> {
        if self.0 {
            self.0 = false;
            Some("totally the last item!!!")
        } else {
            self.0 = true; // D'oh!
            None
        }
    }
}

let mut flaky = Flaky(true);
assert_eq!(flaky.next(), Some("totally the last item!!!"));
assert_eq!(flaky.next(), None);
assert_eq!(flaky.next(), Some("totally the last item!!!"));

let mut not_flaky = Flaky(true).fuse();
assert_eq!(flaky.next(), Some("totally the last item!!!"));
assert_eq!(flaky.next(), None);
assert_eq!(flaky.next(), None);

// el adaptador fuse es probablemente mas util en codigo generico que necesita trabajar con
// iteradores de un origen incierto. Para asegurar el correcto comportamiento
//
// Iteradores reversibles y rev
// Algunos iteradores son capaces de dibujar items desde el final de una secuencia. Podemos
// iterar de forma inversa usando el adaptador `rev`. Por ejemplo un iterador sobre un vector puede
// solo dibujar items desde el final del vector hasta el principio. Un iterador asi puede
// implementar el trait `std::DoubleEndedIterator` el cual extiende `Iterator`

trait DoubleEndedIterator: Iterator {
    fn next_back(&mut self) -> Option<Self::Item>;
}

// `inspect`: Este adaptador es bueno para debuggear pipelines de adaptadores, pero no se usa mucho
// cuando queremos usarlo en produccion. Simplemente aplica un closure a una referencia compartida
// a cada item y entonces pasa el item
// El closure no puede afectar el item, pero podemos hacer cosas como imprimir el contenido de cada
// item o hacer asserts sobre ellos(cosas piolisimas si queremos saber que esta pasando en nuestro
// iterators)
//
// `chain`: Este adaptador agrega un iterador a otro, mas precisamente `i1.chain(i2)` retorna un
// iterator que saca items de i1 hasta que es vaciado y entonces saca items de i2;
// La signatura de `chain` es la siguiente:
//
fn chain<U>(self, other: U) -> some Iterator<Item=Self::Item>
where Self: Sized, U: IntoIterator<Item=Self::Item>;

// en otras palabras podemos encadenar un iterador con cualquier iterable que produzca el mismo
// item
//
let v: Vec<i32> = (1..4).chain(vec![10, 20, 30]).collect();
assert_eq!(v, vec![1, 2, 3, 10, 20, 30]);

// NOTE(elsuizo:2020-05-12): piolisimaaa
// un iterador `chain` es reversible si ambos iteradores a encadenar lo son:
let v: Vec<i32> = (1..4).chain(vec![10, 20, 30]).rev().collect();
assert_eq!(v, vec![30, 20, 10, 3, 2, 1]);

// `enumerate`: este trait agrega un indice que se va aumentando cuando vamos iterando sobre el
// iterador, por ejemplo si un iterador crea los items A, B, C,... entonces con enumerate vamos a
// crear la tupla (A, 1), (B, 2), (C, 3).... Por ejemplo cuando hicimos el ejemplo de el conjunto
// de Mandelbrot que dividiamos a la imagen en 8 bandas horizontales y asignabamos a cada una un
// thread, ese codigo utilizaba `enumerate` para decirle a que thread pertenecia esa banda que
// dividiamos.
// Comenzando con un buffer rectangular de pixels:
let mut pixels = vec![0; colums * rows];

let threads = 8;
let bands_rows = rows / threads + 1;
//...
let bands: Vec<&mut [u8]> = pixels.chunks_mut(band_rows * columns).collect();

// y luego iterabamos sobre las bandas:
for (i, band) in bands.into_iter().enumerate() {
    let top = band_rows * i;
    // start a thread to render rows ...
}

// cada iteracion obtiene un par (i, band) donde `band` es el `&mut [u8]`
//
// `zip`: este adaptador combina dos iteradores en un iterador que produce pares manteniendo un
// valor desde cada iterador, como zipper
//
// NOTE(elsuizo:2020-05-12): notemos que utiliza un range infinito!!!
let v: Vec<_> = (0..).zip("ABCD".chars()).collect();
assert_eq!(v, vec![(0, 'A'), (1, 'B'), (2, 'C'), (3, 'D')]);


// el argumento de zip puede ser cualquier iterable:
use std::ite::repeat;
let endings = vec!["once", "twice", "chicken soup with rice"];
let rhyme: Vec<_> = repeat("going").zip(endings).collect();
assert_eq!(rhyme, [("going", "once"), ("going", "twice"), ("going", "chicken soup whith rice")]);

// `by_ref`: A traves de esta seccion estuvimos insertando adaptadores a los iteradores, una vez
// que hemos hecho esto, podemos tomar el adaptador de nuevo???. Usualmente no porque los
// adaptadores toman la propiedad del iterador que esta corriendo por debajo del capo y no proveen
// un metodo para traerlos de nuevo. El metodo de los iteradores `by_ref` comparte una referencia
// mutable a el iterador entonces podemos aplicar adaptadores a la referencia. Cuando hemos
// terminado consumiendo los items de estos adaptadores, los eliminamos.
// Por ejemplo antes vimos como usar `take_while` y `skip_while` para procesar las lineas de un
// mail, pero que pasa si queremos utilizar los dos usando el mismo iterador, con `by_ref` podemos
// usar como lo hicimos `take_while` para filtrar los headers y cuando este ha terminado tener el
// iterador de nuevo y en el mismo lugar donde lo ha dejado `take_while` para filtrar el mensaje
// que es lo que sigue
//
let message = "To: jimb\r\n\
               From: id\r\n\
               \r\n\
               Ohhhh, donuts!!!\r\n";
let mut lines = message.lines();

println!("Headers: ");
for header in lines.by_ref().take_while(|line| !line.is_empty()) {
    println!("{}", header);
}

println!("\nBody: ");
for body in lines {
    println!("{}", body);
}

// `cloned`: El adaptador cloned toma un iterador que produce referencias y retorna un iterador que
// produce valores clonados desde esas referencias. Naturalmente, el type del referente debe
// implementar `Clone`: por ejemplo:
let a = ['1', '2', '3', '4'];

assert_eq!(a.iter().next(), Some(&'1'));
assert_eq!(a.iter().cloned().next(), Some('1'));

// `cycle`: Este adaptador retorna un iterador que sin final repite la secuencia producida por el
// unico iterador que esta corriendo. Este iterador debe implementar `std::clone::Clone` ya que
// `cycle` puede guardar su estado inicial y reusarse cada vez que el ciclo comienza de nuevo.
// Por ejemplo:
let dirs = ["North", "East", "South", "West"];
let mut spin = dirs.iter().cycle();
assert_eq!(spin.next(), Some(&"North"));
assert_eq!(spin.next(), Some(&"Easth"));
assert_eq!(spin.next(), Some(&"South"));
assert_eq!(spin.next(), Some(&"West"));
assert_eq!(spin.next(), Some(&"North"));
// and son on...
// O podemos hacer sino el famoso ejemplo de fizzbuzzz:(lo hago en un repo aparte asi me queda)
//
//
// Consumiendo iteradores: Vimos como crear, usar y agregarle adaptores a los iteradores pero una
// cosa que nos falta ver mas en detalle es las maneras que hay de consumir a estos ademas de las
// conocidas llamando a el metodo `next()` o en un loop `for`
//
// Simple acumulacion: `count`, `sum` y `product`
//
// El metodo `count` saca los items de un iterador hasta que este retorna `None` y nos dice cuantos
// items ha sacado. Por ejemplo podemos utilizarlo para contar la cantidada de lineas de una
// entrada:
use std::io::prelude::*;

fn main() {
    let stdin = std::io::stdin();
    println!("{}", stdin.lock().lines().count);
}

// luego los metodos `sum` y `product` toman los elementos de un iterador y los suman o los
// multiplican, los cuales deben ser enteros o floats, pero tambien lo podemos extender a otros
// types impl los traits `std::iter:Sum` y `std::iter::Product`
//
//
// `max` y `min`: Estos metodos como su nombre lo indica nos devuelven el maximo valor y el minimo
// valor que un iterador produce. El item del iterador debe impl `std::cmp::Ord` para poder
// utilizarlo (para que puedan comparar unos con otros):
assert_eq!([2, 0, 3, 4, 5].iter().max(), Some(&5));
assert_eq!([2, 0, 3, 4, 5].iter().min(), Some(&0));

// Estos metodos retornan un `Option<Self::Item>` entonces pueden retornar `None` si es que el
// iterador no produce items. Como vimos los types de floats en Rust no implementan `std::cmp::Ord`
// por eso no podemos utilizar estos metodos donde hayan estos items(es por una cuestion de
// seguridad ya que no esta claro que deben hacer estas funciones con los valores NaNs, ignorarlos
// puede ser perligroso (para el lenguaje que no quiere ese Default)) Si sabemos que podemos
// manejar estos valores NaNs podemos utilizar los metodos `max_by` y `min_by` los cuales nos dejan
// la tarea a nosotros de suplir con una funcion que compare los valores(como con `sort_by`)
//
// `max_by` y `min_by`: Como vimos retornan el minimo o el maximo utilizando una funcion de
// comparacion que nosotros le pasamos, por ejemplo para el caso de floats
//
use std::cmp::{ParitialOrd, Ordering};

// cmp dos f64. Panic si nos devuelve un NaN
fn cmp(lhs: &&f64, rhs; &&f64) -> Ordering {
    lhs.partial_cmp(rhs).unwrap()
}

let numbers = [1.0, 2.3, 4.5];
assert_eq!(numbers.iter().max_by(cmp), Some(&4.5));
assert_eq!(numbers.iter().min_by(cmp), Some(&1.0));

// y si encuentra un NAN paniquea!!!
//
// la referencia doble en la funcion de comparacion es porque `numbers.iter()` produce una
// referencia a los elementos de el array y entonces `max_by` y `min_by` pasan la referencia del
// clousure a el item del iterador
//
//
// `max_by_key` y `min_by_key`: Nos permite elegir el maximo y el minimo valor que ocurre en un
// iterador que es determinado por un closure que le pasamos. El closure puede elegir algun field
// del item o hacer alguna cuenta sobre el item(para ver si es max o min) ya que muy a menudo
// estamos interesados en datos asociados a un minimo o un maximo y no solo el dato en si. Por eso
// dice que estas funciones en si son mas utilies que las anteriores. Sus signaturas son las
// siguientes:
//
fn min_by_key<B: Ord, F>(self, f: F) -> Option<Self::Item>
where Self: Sized, F: FnMut(&Self::Item) -> B;

fn max_by_key<B: Ord, F>(self, f: F) -> Option<Self::Item>
where Self: Sized, F: FnMut(&Self::Item) -> B;

// esto es un dado closure que toma un item y retorna cualquier type que impl `Ord<B>` retorna el
// item por el cual el closure retorna el max y el min o `None` si no se ha producido ningun
// resultado.
// Por ejemplo si queremos escanear una hash table de ciudades para hayar las ciudades con el
// menor poblacion, podemos escribir:
use std::collections::HashMap;

let mut populations = HashMap::new();
populations.insert("Portland", 587_776);
populations.insert("Fossil", 449);
populations.insert("Greenhorn", 2);
populations.insert("Boring", 7_762);
populations.insert("The Dalles", 16_437);

assert_eq!(populations.iter().max_by_key(|&(_name, pop)| pop), Some((&"Portland", &587_776)));
assert_eq!(populations.iter().min_by_key(|&(_name, pop)| pop), Some((&"Greenhorn", &2)));

// el closure `|&(_name, pop)| pop` es aplicado a cada item que el iterador produce y retorna el
// valor para usarlo como comparacion(en este caso la poblacion de una ciudad)
// TODO(elsuizo:2020-05-13): no me quedo claro como es que saca el valor de el iter y compara en
// este ejempo

// Comparando sequencias de items
//
// Podemos usar los operadores `<` y `==` para comparar strings, vectores y slices, aunque Rust no
// soporta todavia comparar a iteradores directamente, pero provee algunos metodos que son
// parecidos, por ejemplo:
//
let packed = "Helen of Troy";
let spaced = "Helen of   Troy";
let obscure = "Helen of Sandusky"; // nice person, just no famous

assert!(packed != spaced);
assert!(packed.split_whitespace().eq(spaced.split_whitespace()));

// true porque  '' < 'o'
assert!(spaced < obscure);

// true porque 'Troy' > 'Sandusky'
assert!(spaced.split_whitespace().gt(obscure.spit_whitespace()));

// las llamadas a `split_whitespace()` retornan un iterador sobre las palabras que son separadas
// por espacios en blancos, usando los operadores `gt` y `eq` podemos hacer que este iterador
// compare palabra a palabra en lugar de hacer `char` a `char`. Esto es posible gracias a que &str
// y String implementan `PartialOrd` y `PartialEq`
//
// `any` y `all`: estos methodos aplican un closure a cada item que el iterador y retornan un
// `true` si el closure retorna `true` para cualquiera de los items, o para todos los items:
//
let id = "Iterator";

// alguno es mayuscula??? ---> true
assert!(id.chars().any(char::is_uppercase()));
// todos son mayuscula??? ---> false
assert!(!id.chars().all(char::is_uppercase()));

// Lo interesante de estos metodos es que solo consumen los items que necesitan para determinar una
// respuesta, por ejemplo en el primero solo necesita consumir el primero!!!(esta es una ventaja
// evidente de los iteradores sobre los loops a lo fuerza bruta)
//
// `position` `rposition` y `ExactSizeIterator`
//
// El metodo `position` aplica un closure a cada item que nos devuelve el iterador y retorna el
// indice del primer item para el cual el closure retorna `true`, mas precisamente retorna un
// `Option<index>`: si el closure no retorna true para ningun item entonces `position` retorna
// `None`, deja de sacar elementos de el iterador una vez que la condicion se ha cumplido. Por
// ejemplo:
//
let text = "Xerxes";
assert_eq!(text.chars().position(|c| c == 'e'), Some(1));
assert_eq!(text.chars().position(|c| c == 'z'), None);

// el metodo `rposition` es lo mismo lo unico que cambia es que comienza a buscar desde la derecha,
// por ejemplo:
//
let bytes = b"Xerxes";
// recordemos que comienza desde la izquierda y como hay dos e la que primero encuentra es la que
// esta en 4!!!
assert_eq!(bytes.iter().rposition(|&c| c == b'e'), Some(4));
assert_eq!(bytes.iter().rposition(|&c| c == b'X'), Some(0));

// para utilizar a este metodo necesitamos un iterador que sea reversible y tambien necesitamos que
// el iterador tenga un size conocido un iterador que tiene un size conocido si implementa el trait
// `std::iter::ExactSizeIterator`

pub trait ExactSizeIterator: Iterator {
    fn len(&self) -> usize {...};
    fn is_empty(&self) -> bool {...};
}

// el metodo `len` retorna el numero de items que quedan por recorrer y `is_empty()` retorna `true`
// si la iteracion es completa
//
// `fold`: Este metodo es una herramienta general para acumular ciertos resultados sobre una
// sequencia entera de items que un iterador produce. Dado un valor inicial que podemos llamar el
// acumulador y un closure, `fold` repetidamente aplica el closure a el acumulador actual y el
// proximo item desde el iterator. El valor que el closure retorna es tomado como el nuevo
// acumulador, que sera pasado al closure con el proximo item, el ultimo valor que toma el
// acumulador es lo que fold retorna. Como sabemos del fucking Haskell podemos escribir la mayoria
// de los metodos para consumir un iterador con `fold`.
let a = [5, 6, 7, 8, 9, 10];
assert_eq!(a.iter().fold(0, |n, _| n + 1), 6);      // count
assert_eq!(a.ter().fold(0, |n, i| n + i), 45);      // sum
assert_eq!(a.iter().fold(1, |n, i| n * i), 151200); // product

// max
assert_eq!(a.iter().fold(i32::min_value(), |m, &i| std::cmp::max(m, i)), 10);

// el metodo `fold` tiene la siguiente signatura:
//
fn fold<A, F>(self, init: A, f: F) -> A
where Self: Sized, F: FnMut(A, Self::Item) -> A;

// aca `A` es el type del acumulador, el argumento es tambien un `A` como el valor de retorno del
// closure y el primer parametro que le pasamos al acumulador
// Notemos que los valores del acumulador han sido ingresados y sacados del closure, entonces
// podemos usar fold con types de acumuladores que no permitan copiarse
let a = ["pack", "my", "box", "with", "five", "dozen", "liquors", "jugs"];
let panagram = a.iter().fold(String::new(), |mut s, &w| {s.push_str(w); s});
// NOTE(elsuizo:2020-05-13): hizo esto y le dio lo mismo no se que quizo demostrar
//
//
// `nth`: Este metodo toma un indice `n`, se saltea ese numero de items y nos devuelve el proximo
// item, o `None` si la secuencia termina antes que ese punto. Llamar a `nth(0)` es lo mismo que
// llamar a `next()`
//
//
// `last`: Este metodo consume los items hasta que el iterador retorna `None` y luego retorna el
// ultimo item. Si el iterador no produce items entonces retorna `None`. Su signatura es la
// siguiente:
fn last(self) -> Option<Self::Item>;

// notemos que si tenemos un iterador que es reversible y no necesitamos consumir todo el iterador
// podemos hacer lo mismo pero mucho mas eficientemente con `iter.rev().next()`
//
//
// `find`: El metodo find saca items de un iterador retornando el primer item por el cual es
// verdadera la condicion que nos devuelve un closure que le pasamos, o `None` si la secuencia
// finalliza antes de que encuentre un valor apropiado, su signatura es la siguiente:
fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
where Self: Sized, P: FnMut(&Self::Item) -> bool;

// Construyendo colecciones: `collect` y `FromIterator`
// Como vimos cuando hacemos `collect` ponemos todos los items en un vector, pero collect no solo
// es especifico para generar vectores, de hecho podemos construir cualquier tipo de collection
// desde la libreria std de Rust, siempre que el iterador produzca un item del type apropiado
// por ejemplo:
//
use std::collections::{HashSet, BtreeSet, LinkedList, HashMap, BTreeMap};

let args: HashSet<String> = std::env::args().collect();
let args: BTreeSet<String> = std::env::args().collect();
let args: LinkedList<String> = std::env::args().collect();

// colectar un map requiere un par (keys, value), entonces para este
// ejemplo, hacer un `zip` de la secuencia de strings con una secuencia
// de enteros
let args: HashMap<String, usize> = std::env::args().zip(0..).collect();
let args: BTreeMap<String, usize> = std::env::args().zip(0..).collect();
// y asi con los demas
//
// Naturalmente collect en si mismo no sabe como construir todos estos types
// mas bien que para que una coleccion com `Vec` o `HashMap` pueda conocer como
// construirse a si misma desde un iterador debe impl `std::iter::FromIterator`

trait FromIterator<A>: Sized {
    fn from_iter<T: IntoIterator<Item=A>>(iter: T) -> Self;
}
// Si una coleccion implementa `FromIterator<A>` entonces su metodo
// estatico `from_iter` genera un valor de ese type desde un iterable
// produciendo items del type `A`
//
//
// El trait `Extend` si un type implementa el `std::iter::Extend` entonces su metodo extendido
// agrega un item iterable a la coleccion:
//
let mut v: Vec<i32> = (0..5).map(|index| 1 << index).collect();
v.extend(&[31, 57, 99, 163]);
assert_eq!(v, &[1, 2, 4, 8, 16, 31, 57, 99, 163]);

// todas las collecciones de la libreria estandar impl este trait, como por ejemplo `String`. Pero
// las colecciones que tienen un length fijo no como son los arrays y slices, la definicion del
// trait es la siguiente:
trait Extend<A> {
    fn extend<T>(&mut self, iter: T)
        where T: IntoIterator<Item=A>;
}

// `partition`: este metodo divide a item de un iterador en dos colecciones usando un closure para
// decidir donde poner a cada item(en cual de los dos en que dividio)
//
let things = ["doorknoob", "mushroom", "noodle", "giraffe", "grapefruit"];

let (living, nonliving) = (Vec<&str>, _) = things.iter().partition(|name| name.as_bytes()[0] & 1 != 0);
assert_eq!(living, vec!["mushroom", "giraffe", "grapefruit"]);
assert_eq!(nonliving, vec!["doorknoob", "noodle"]);

// la signatura de partition es la siguiente:
//
fn partition<B, F>(self, f: F) -> (B, B)
    where Self: Sized,
          B: Default + Extend<Self::Item>,
          F: FnMut(&Self::Item) -> bool;

// vemos que para utilzar este metodo el iter debe impl `Extend` y `Default`
//
// Implementando nuestros propios `Iterators`
//
// Podemos implementar el trait para nuestros types haciendo que todos los adaptadores esten
// disponibles para ellos. Vamos a ver un simple ejemplo, supongamos que tenemos un type del stilo
// range, supongamos que tenemos el siguiente type:
struct I32Range {
    start: i32,
    end: i32
}

// iterar sobre un `I32Range` requiere dos estados: el valor actual y el limite en el cual la
// iteracion deberia finalizar. Podemos aprovechar esto para implementar el type usando `start`
// como el proximo valor y `end` como el limite. Entonces podemos impl el iterador asi:
//
impl Iterator fot I32Range {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        if self.start >= self.end {
            return None;
        }
        let result = Some(self.start);
        self.start += 1;
        result
    }
}

// por supuesto que cuando utilizamos un loop `for` usa `IntoIterator::into_iter` para convertir su
// operando en un iterador. Pero por suerte la libreria estandar provee una implementacion por
// default para cada type que implemente `Iterator`, por eso `I32Range` esta listo para usarse en
// un loop `for`
// Pero todos los casos no son tan simples como este, veamos lo que sucede si queremos hacer una
// implementacion de nuestro type `BinaryTree`, la manera clasica de "caminar" un arbol binario es
// recursivamente, usando el stack de llamado de funciones para saber donde estamos en el arbol y
// que nodos hemos visitado o tenemos que visitar, pero cuando implementamos `Iterator` para
// `BinaryTre<T>` cada llamada a `next()` debe producir exactamente un valor de retorno. Para
// mantener una cuenta de los nodos que aun no hemos producido, el iterador debe mantener una
// cuenta por sus propios medios de esto.
//
//
// Cap 16: Colecciones
// Las colecciones que tenenmos disponibles desde la libreria estandar son:
//
// `Vec<T>`: Es un array que puede crecer de tamanio que se almacena en la memoria "heap" cuyos
// elementos son todos del type `T`.
//
// `VecDeque<T>`: Es como `Vec<T>` pero mejor en situaciones que tenemos que utilizar una cola
// FIFO, ya que soporta eficientemente agregar y remover valores desde el frente de la lista como
// asi tambien desde la cola. Esto tiene como desventaja que las operaciones que hagamos con esta
// estructura de datos van a ser mas lentas.
//
// `LinkedList<T>`: Soporta acceso rapido en el frente de la queue y en la cola, como `VecDeque<T>`
// lo que permite que hagamos concatenaciones rapidas. Sin embargo en general, `LinkedList<T>` es
// mas lento que un `Vec<T>` y que `VecDeque<T>`.
//
// `BinaryHeap<T>`: Es una cola de prioridad. Los valores en esta estructura son organizados
// entonces es siempre eficiencte para encontrar y remover los valores maximos(ya que van a estar
// siempre en la cola o en el frente).
//
// `HashMap<K, V>`: es una tabla de valor--->key. Por ello cuando buscamos un valor por su key es
// super rapido. Las entradas son guardadas en un orden arbitrario.
//
// `BTreeMap<K, V>`: es como un `HashMap<K, V>` pero este guarda los elemento ordenados por el
// "key". Por ejemplo: `BTreeMap<String, i32>` guarda sus entradas ordenando los `String`
// comparandolos a menos que necesitemos esta funcionalidad es mejor utilizar un `HashMap<K, V>`
//
// `HashSet<T>`: Es un conjunto de valore que son del type `T`, agregar y sacar valores es rapido y
// es rapido preguntar cuando un determinado valor esta en el conjunto o no.
//
// `BTreeSet<T>`: es como `HashSet<T>` pero mantiene los elementos ordenados por valor, de nuevo si
// no lo necesitamos siempre son mas rapidos los que no mantienen el orden
//
//
// `Vec<T>`: La manera mas facil de crear un vector es usando el macro `vec!`:
//
let mut numbers: Vec<i32> = vec![];

// creamos un vector con contendido
let words = vec!["step", "on", "no", "pets"];
let mut buffer = vec![0u8; 1024]; // 1024 bytes de valor 0x00000000

// Como vimos en el capitulo 4 un vector tiene 3 campos: el largo, la capacidad y un puntero a una
// memoria en el heap donde los elementos son guardados
//
// Accediendo a los elementos: como sabemos para acceder a los elementos solo tenemos que
// indexarlos(siempre recordando que los indices comienzan en 0)
let first_line = &line[0];

// get a copy of a element
let fifth_number = number[4]; // requiere Copy
let secon_line   = line[1].clone(); // requiere Clone

// get a reference to a slice
let my_ref = &buffer[4..12];

// get a copy of a slice
let my_copy = buffer[4..12].to_vec(); // requiere Clone

// todos los anteriores van a paniquear si el inice esta fuera de rango
//
// Muchos metodos proveen un acceso facil a un elemento en particular de un vector o de un
// slice(notemos que todos los metodos de los slides estan disponibles para arrays y para vectores)
//
// `slice.first()`: retorna una referencia a el primer elemento de un slice, si es que tiene...El
// type que retorna es un `Option<&T>` entonces retorna `None` si es que el slice esta vacio y
// `Some(&slice[0])` si no esta vacio
//
if let Some(first_item) = v.first() {
    println!("hay elementos en el vector v: {:}", first_item);
}

// `slice.last()`: es similar al anterior pero retorna una referencia al ultimo elemento
//
// `slice.get(index)`: retorna un `Some()` a una referencia a un `slide[index]` si es que existe.
// Si el slice tiene menos de `index + 1` elementos retorna un tremendo `None`
// recordemos que no podemos indexar los elementos de una coleccion con `u32` o `ixx` el unico
// type que esta permitido para indexar es `usize`(para que funcione en todas las plataformas!!!)
//
// `slice.first_mut()`, `slice.last_mut()` y `slice.get_mut(index)`: son variaciones de los
// anteriores que piden prestado referencias mutables
//
// `slice.to_vec()`: clone a todo el slice y retorna un vector nuevo, el metodo esta disponible
// solo si los elementos son clonables
//
// Iteracion: Los vectores y los slices son iterables tanto por valor como por referencia,
// siguiendo el patron que vimos cuando vimos las implementaciones de `IntroIterator`:
//  - Iterar sobre un `Vec<T>` produce items del type `T`. Los elementos son "movidos" del vector
//  uno por uno consumiendolo(ah re)
//  - Iterar sobre uno de estos types: `&[T; N]`, `&[T]` o sobre `&Vec<T>` osea una referencia a un
//  array a un slice o a un vector produce items del type `&T`, o sea referencias a los elementos
//  individuales, los cuales NO son movidos
//  - Iterar sobre un valor del type `&mut [T; N]`, `&mut [T]` o `&mut Vec<T>` produce items del
//  type `&mut T`

/// Creciendo y achicando vectores
// El tamanio de un array, slice o vector es el numero de elementos que contienen:
//  - `slice.len()`: devuelve el tamanio de el slice como un `usize`
//  - `slice.is_empty()`: es `true` si el slice no contiene elementos(o sea que `slice.len() == 0`)
// Recordemos que los siguientes metodos no estan disponibles para slices o arrays ya que como
// sabemos son staticos
// Todos los elementos de un vector son almacenados en la memoria "heap", la capacidad del vector
// es el numero maximo de elementos pueden caber en este "chunk". `Vec` (normalmente) maneja la
// capcidad automaticamente por nosotros, existen unos pocos metodos que se relacionan con la
// capacidad explicitamente:
//  - `Vec::with_capacity(n)`: crea un nuevo vector con capacidad `n`
//  - `vec.capacity()`: retorna la capacidad de un vector, como `usize` es siempre verdadero que
//  `vec.capacity() >= vec.len()`
//  - `vec.reserve(n)`: Se asegura que tengamos por lo menos ese espacio en memoria para `n`
//  elementos, esto es `vec.capacity()` es al menos `vec.len() + n`, si ya hay el espacio
//  suficiente este metodo no hace nada, sino alloca un buffer mas grande y mueve el contenido del
//  vector a ese nuevo espacio en memoria
//  - `vec.reserve_exact(n)`: es como el anterior pero le dice a `vec` que no alloque ningun byte
//  extra de capacidad por un futuro crecimiento del vector mas alla de `n` despues de la
//  operacion `vec.capacity()` es axactamente `vec.len() + n`
//  - `vec.shrink_to_fit()`: intenta liberar un extra de memoria si `vec.capacity()` es mayor que
//  `vec.len()`
//
//  Como sabemos tenemos metodos para sacar o poner elementos en un vector:
//  - `vec.push(value)`: agrega un elemento al vector en el final de el(notemos que toma a el valor
//  por valor y no por referencia)
//  - `vec.pop()`: remueve un elemento de un vector(el ultimo). El valor de retorno es un
//  `Option<T>` ya que puede ser que el vec este vacio(devuelve `None`)
//
//  Notemos que `.push()` toma a su argumento por valor, no por referencia.
//  Los siguientes dos metodos agregan o remueven un valor en cualquier lugar del vector:
//
//  - `vec.insert(index, value)`: inserta el valor dado en la posicion `vec[index]` paniquea si
//  `index > vec.len()`
//  - `vec.remove(index)`: remueve y retorna `vec[index]`
//
// A mas grande que sea el vector mas lenta seran estas operaciones, si nos vemos envueltos
// haciendo `vec.remove(0)` tooodo el tiempo entonces deberiamos utilizar `VecDeque`
// Tenemos tres metodos para cambiar el largo de un vector a un valor especifico:
//  - `vec.resize(new_len, value)`: Setea el largo del vector a un `new_len`. Si esto hace que el
//  largo del vector crezca, copias del nuevo valor pasado son agregados para rellenar el nuevo
//  espacio, el elemento debe impl `Clone`
//  - `vec.truncate(new_len)`: reduce el largo del vector a `new_len` "tirando" cualquier elemento
//  que este en el rango de `vec[new_len..]`
//  - `vec.clear()`: remueve todos los elementos de un vector, es lo mismo que hacer
//  `vec.truncate(0)`
//
//  Tenemos cuatro metodos para agregar un remover un valor o varios a la vez:
//  - `vec.extend(iterable)`: agrega en el final del vector todos los items desde un iterable dado
//  en orden. Es como una version multivalor de `push()`. El argumento iterable puede ser
//  cualquiera que implemente `IntoIterator<Item=T>`
//  - `vec.split_off(index)`: es como `vec.truncate(index)` exepto que este retorna un `Vec<T>`
//  conteniendo los valores que fueron removidos desde el final del `vec`. Es como una version
//  multivalor de `pop()`
//  - `vec.append(&mut vec2)`: Donde `vec2` es otro vector del type `Vec<T>`, mueve todos los
//  elementos desde `vec2` a `vec`. Despues de esta operacion `vec2` estara vacio.
//  - `vec.drain(range)`: Donde `range` es un valor del type `Range` como `1..4`, remueve los
//  elementos que estan en el rango `vec[range]` desde vec y retorna un iterador sobre los
//  elementos removidos
//
//  Hay tambien algunos metodos para remover selectivamente alguno de los elementos de un vector:
//  - `vec.retain(test)`: remueve todos los elementos que no pasan un dado test. El test es pasado
//  como un closure que implemente: `FnMut(&T)->bool`, para cada elemento de el vector esto llama a
//  el closure `test(&element)` y si retorna `false` el elemento se remueve del vector y se tira el
//  valor, sin comparar sus performance es como escribir el siguiente codigo:
//  NOTE(elsuizo): cual es mas rapido???
vec = vec.into_iter().filter(test).collect();

//  - `vec.dedup()`: "drops" elementos repetidos. Es como la utilidad de Unix `uniq`. Escanea `vec`
//  en lugares donde elementos adyacentes son iguales y "tira" los valores que son iguales, tal que
//  uno solo queda:

let mut byte_vec = b"Misssssssipppi".to_vec();
byte_vec.dedup();
assert_eq!(&byte_vec, b"Misipi");

// Notemos que solo remueve los duplicados que estan uno adyacente al otro y no todos los
// repetidos. Para eliminar los duplicados tenemos tres opciones: ordenar el vector antes de
// llamar `.dedup` mover los datos a un conjunto o "set" o (mantener los elementos en su estado
// original) usando el truco de `.retain()`:
let mut byte_vec = b"Misssssssipppi".to_vec();
let mut seen = HashSet::new();
byte_vec.retain(|r| seen.insert(*r));
assert_eq!(&byte_vec, b"Misp");
// cuando intentamos insertar un elemento que ya esta en el conjunto este no nos deja por eso
// funciona esto.
//
// - `vec.dedup_by(same)`: Es lo mismo que el anterior, pero usa una funcion o closure `same(&mut
// elem1, &mut elem2)` en lugar del operador `==` para checkear cuando dos elementos deben ser
// considerados iguales
//
// - `vec.dedup_by_key(key)`: Es lo mismo que `vec.dedup()` pero el trata a los elementos como
// iguales si `key(&mut elem1) == key(&mut elem23)` Por ejemplo, si los errores los tenemos en un
// `Vec<Box<Error>>` podemos escribir:
// remover errores con mensajes redundantes...
errors.dedup_by_key(|err| err.description().to_string());

// De todos los metodos que vimos en esta seccion solo `resize()` clona valores, los otros mueven
// los valores de un lado a otro
//
//
// Uniendo: Dos metodos que trabajan sobre array de arrays, por el cual decimos cualquier array,
// slice o vector cuyos elementos en si sean arrays, slices o vectores:
//
// - `slice.concat()`: retorna una nuevo vector hecho por la concatenacion de todos los slices.
// - `slice.join(&separator)`: es lo mismo exepto que una copia del valor que le pasamos como
// `separator` es insertada entre los slices
assert_eq!([[1, 2], [3, 4], [5, 6]].join(&0), vec![1, 2, 0, 3, 4, 0, 5, 6]);

// Dividiendo: Es facil obtener muchas referencias no mutables en un array, slice o vector de una
// sola vez:
let v = vec![0, 1, 2, 3];
let a = &v[i];
let b = &v[j];

let mid = v.len() / 2;
let front_half = &v[..mid];
let back_half = &v[mid..];

// pero obtener muchas referencias mutables no es ni deberia ser facil
let mut v = vec![0, 1, 2, 3];
let a = &mut v[i];
let b = &mut v[j]; // ---> error no podemos prestar a v mas de una vez porque que mutable!!!
// Rust prohibe esto porque si `i==j` entonces `a` y `b` podrian tener una referencia al mismo
// entero que es una violacion a los principios de Rust. Rust tiene muchos metodos que pueden
// prestar referencias mutables de dos o mas partes de un array, slice o vector, que a diferencia
// del codigo anterior son seguros, ya que por disenio parten los datos en regiones de memoria que
// no se solapan. Muchos de estos metodos son utiles para trabajar tambien con slices-no-mutables
// ya que hay una version mutable y no mutable de cada uno.
//
// - `slice.iter()` y `slice_iter_mut()`: producen una referencia a cada elemento de un slice
// - `slice.split_at(index)` y `slice_split_at_mut(index)`: parte a un slice en dos, retornando un
// par. Es equivalente a hacer: `(&slice[..index], &slice[index..])` estos metodos paniquean si el
// index esta fuera de
// - `slice.split_first()` y `slice.split_first_mut()`: Tambien retornan un par: una referencia al
// primer elemento (`slice[0]`) y un slice que referencia a todo el resto(`slice[1..]`). El type de
// retorno es `Option<(&T, &[T]>)` y el resultado es un `None` si el slice esta vacio
// - `slice.split_last()` y `slice.split_last_mut()` son los analogos pero sacan el ultimo elemento
// en lugar del primero
// - `slice.split(is_sep)` y `slice.split_mut(is_sep)` separan un slice en uno o mas subslices,
// usando un closure `is_sep` para saber donde tiene que separar. Retorna un iterator sobre los
// subslices
// - `slice.splitn(n, is_sep)` y `slice.splitn_mut(n, is_sep)` son lo mismo que el anterior pero
// produce a lo sumo `n` subslices. Despues de los primeros `n-1` slices que son encontrados
// `is_sep` no es llamado de nuevo. El ultimo subslice contiene todos los elementos restantes
// - `slice.rsplitn(n, is_sep)` y `slice.rsplitn_mut(n, is_sep)` son como los anteriores salvo que
// el slice es escaneado en orden reverso.
// - `slice.chuncks(n)` y `slice.chuncks_mut(n)`: Retornan un iterador sobre subslices que no se
// solapan de largo `n`. Si `slice.len()` no es un multiplo de `n`, entonces el ultimo subslice
// tendra un largo menor que `n`
// - `slice.windows(n)`: Retorna un iterador que se comporta como una ventana deslizante sobre los
// datos en el slice. Produce subslices que se abarcan `n` elementos consecutivos del slice. El
// primer valor producido es `&slice[0..n]` el segundo `&slice[1..n+1]` y asi siguiendo
// Si el `n` es mayor que el `slice.len()` entonces no se producen slices si `n==0` entonces
// paniquea.
// por ejemplo si tenemos un vector con los dias del mes `days.len() == 31` entonces podemos
// producir los dias de una semana haciendo `days.window(7)`. Tambien un sliding window de 2 es
// bueno cuando queremos explorar como cambia una serie de datos desde un punto al otro:
let changes = daily_high_temperatures.windows(2).map(|w| w[0] - w[1]).collect::<Vec<_>>();
// NOTE(elsuizo:2020-05-19): tambien nos puede servir cuando queremos hacer una discretizacion
//
// Intercambiando: Existe un metodo conveniente para intercambiar dos elementos:
// `slice.swap(i, j)`: intercambi los elementos que estan en `slice[i]` `slice[j]` y los vectores
// tienen un metodo relacionado para remover eficientemente cualquier elemento
// - `vec.swap_remove(i)`: remueve y retorna `vec[i]`. Esto es como hacer `vec.remove(i)` exepto
// que en lugar de deslizar el resto de los elementos del vector para cerrar el gap que queda,
// simplemente mueve el ultimo elemento para cerrar ese gap, esto es util cuando no nos interesa el
// orden despues de la operacion
//
// Ordenando y buscando: Los slices ofrecen tres metodos para ordenar:
// - `slice.sort()`: Ordena los elementos en orden creciente. Este metodo solo lo podemos usar
// cuando los elementos imp `Ord`
// - `slice.sort_by(cmp)`: Ordena los elementos de un slice usando una funcion o clusure `cmp` para
// especificar el orden. `cmp` debe implementar `Fn(&T, &T)->std::cmp::Ordering`
//
students.sort_by(|a, b| a.last_name.cmp(&b.last_name));

// Si queremos ordenar por un solo campo, usando un segundo campo como el que decide en un empate,
// podemos comparar tuplas:
students.sort_by(|a, b| {
    let a_key = (&a.last_name, &a.first_name);
    let b_key = (&b.last_name, &b.first_name);
    a_key.cmp(&b_key)
});

// `slice.sort_by_key(key)`: Ordena los elementos de un slice en orden creciente dado un `key` que
// decide, dada la funcion o clusure `key`. El type de `key` debe implementar: `Fn(&T)->K` donde `K:Ord`
// Esto es util cuando `T` contiene uno o mas campos desordenados, o sea que puede ser ordenado de
// muchas maneras
students.sort_by_key(|s| s.grade_point_average());

// Si queremos ordenar en orden reverso, podemos usar `sort_by` con un closure `cmp` que permute
// los dos argumentos. Tomando los argumentos `|b, a|` en lugar de `|a, b|` produce efectivamente
// el orden opuesto. O solo podemos llamar a la funcion `reverse()` despues de ordenar:
// - `slice.reverse()`: invierte un slice "in-place"
// Una vez que el slice es ordenado podemos buscar en el eficientemente:
// - `slice.binary_search(&value)` `slice.binary_search_by(&value, cmp)` y
// `slice.binary_search_by_key(&value, key)` todos buscan por el `value` en el slice notemos que el
// valor es pasado por referencia, el type de retorno es `Result<usize, usize>` que retorna un
// `Ok(index)` si el `slice[index]` es igual al `value`. si no existe dicho valor en el slice
// entonces retorna un `Err(insertion_point)` tal que si insertamos un valor an `insertion_point`
// pueda preservar el orden del slice
//
// Existe un metodo para buscar en un vector que no esta ordenado:
//
// - `slice.contains(&value)`: retorna `true` si algun elemento del slice es igual al `value`. Esto
// simplemente busca elemento a elemento del slice hasta que uno es encontrado. Nuevamente, `value`
// es pasado por referencia.
// Para encontrar el lugar en que se encuentra un valor en un slice, podemos usar un iterator:
slice.iter().position(|x| *x == value);

// Esto retorna un `Option<usize>`
//
//
// Comparando slices: Si un type `T` suporta el operador `==` y `!=` (o sea que implementa
// `PartialEq`) entonces los arrays como `[T; N]` los slices como `[T]` y vectores como `Vec<T>`
// los soportan tambien(o sea elemento a elemento y los length son iguales), lo mismo para los
// operadores de `std::cmp::PartialOrd`
//
// Dos metodos para realizar comparaciones entre slices son:
// - `slice.starts_with(other)`: retorna `true` si el slice comienza con una secuencia de valores
// que son iguales a los elementos de otro slice:
assert_eq!([1, 2, 3].starts_with(&[1, 2]), true);
assert_eq!([1, 2, 3, 4].starts_with(&[2, 3]), false);

// - `slice.ends_with(other)`: es similar pero chequea en el final del slice
//
assert_eq!([1, 2, 3, 4].ends_with(&[3, 4]), true);


// Elementos random:
// Como sabemos los numeros aleatorios no estan en la libreria estandar, el crate `rand` los
// provee, el cual provee dos metodos para generar numeros aleatorios sobre arrays, slices o
// vectors
// - `rng.choose(slice)`: retorna una referencia a los elementos random de un slice. Como
// `slice.first()` y `slice.last()` esto retorna un `Option<&T>` que es `None` solo si el slice
// esta vacio
// - `rng.shuffle(slice)`: Reordena aleatoriamente los elementos de un slice "in-place". El slice
// debe ser pasado por referencia
// Estos metodos necesitan un generador de numeros aleatorios para poder llamarlos. Afortunadamente
// es facil:
use rand::{Rng, thread_rng};

// FIXME(elsuizo:2020-05-19): esto no anda mas parece, que el codigo es viejo
thread_rng().shuffle(&mut my_vec);


// `VecDeque<T>`: Mientras que `Vec<T>` soporta agregar eficientemente elementos y removerlos solo
// al final. Cuando un programa necesita un lugar para guardar valores que estan esperando
// "on-line". `Vec` puede ser lento. `std::collections::VecDeque<T>` es una `queue` de doble
// entrada, esta soporta agregar y remover en el principio y en el final eficientemente.
// - `deque.push_front(value)`: agrega un valor en el frente de la cola
// - `deque.push_back(value)`: agrega un valor en el final de la cola. (este metodo es usado mucho
// mas que `push_front()`, porque una convencion para las queues es que los valores se agregan por
// atras y se remueven por el frente, como gente que espera en linea)
// - `deque.pop_front()`: remueve y retorna el elemento que esta en el frente de la queue, retorna
// un `Option<T>` que es `None` si la queue esta vacia, como `vec.pop()`
// - `deque.pop_back()`: remueve y retorna el elemento que esta en el fondo de la cola y retorna un
// `Option<T>`
// - `deque.front()` y `deque.back()`: Trabajan como `vec.first()` y `vec.last()`. Ellos retornan
// una referencia al elemento final de la cola o al primer elemento de la cola, en forma de
// `Option<&T>` que es un `None` si la cola esta vacia
// - `deque.front_mut()` y `deque.back_mut()` trabajan como `vec.first()` y `vec.last()` retornando
// un `Option<&mut T>`
//
//
// `LinkedList<T>`: Una lista enlazada es otra manera de almacenar secuencia de valores. Cada valor
// es guardado en un espacio de memoria distinto en la heap. `std::collections::LinkedList<T>` es
// una lista enlazada doble y soporta un subconjunto de metodos de `VecDeque`
// Por ahora las ventajas que tienen sobre `VecDeque` es que combinar dos listas es muy rapido
//
// `BinaryHeap<T>`: es una coleccion cuyos elementos son guardados de manera libre entonces los
// valores grandes siempre estan el el frente de la queue. Aqui ponemos cuales son los tres usos
// mas comunes de los metodos de `BinaryHeap`:
//
//  - `heap.push(value)`: Agrega un valor al el heap
//  - `heap.pop()`: remueve y retorna el valor mas grande del heap. Retorna un `Option<T>` que es
//  `None` si el heap esta vacio
//  - `heap.peek()`: Retorna una referencia a el valor mas grande en la heap. El valor de retorno
//  es `Option<&T>`
//  `BinaryHeap` tambien soporta un subconjunto de metodos de `Vec`, incluyendo `BinaryHeap::new()`
//  `len()`, `copacity()`, `clear()` y `append(&mut heap2)`, por ejemplo si llenamos una
//  `BinaryHeap` con muchos numeros:
use std::collections::BinaryHeap;

let mut heap = BinaryHeap::from(vec![2, 3, 8, 6, 9, 5, 4]);
// entonces el numero 9 esta en el frente de la heap
assert_eq!(heap.peek(), Some(&9));
assert_eq!(heap.pop(), Some(9));

// cuando removimos el 9 ahora el que estara en el frente sera el 8 y asi sucesivamente
assert_eq!(heap.pop(), Some(8));
assert_eq!(heap.pop(), Some(6));
assert_eq!(heap.pop(), Some(5));

// Pero `BinaryHeap` no esta limitado solo a numeros, podemos guardar cualquier type que impl
// `std::cmp::Ord`. Esto la hace util para que trabaje como queue. Podemos definir una struct
// tarea que impl `Ord` como base para usarla como prioridad de la tarea, entonces las tareas que
// tengan mayor prioridad estaran en el frente de la cola y siempre que hagamos un `pop()`
// estaremos sacando la que tiene mayor prioridad!!!, si bien tiene implementado un `iter()` los
// elementos que saca son aleatorios, si queremos ir sacando elementos por orden de prioridad
// tendriamos que utilizar un `while`:
while let Some(task) = heap.pop() {
    handle(task);
}

// `HashMap<K, V>` y `BTreMap<K, V>`: Un `map` es una coleccion de pares de valores (key-value) que
// los llamamos entradas, en la cual no hay dos entradas que tengan el mismo "key" y las entradas
// se mantienen organizadas tal que si tenemos una key podemos eficientemente ver el
// correspondiente valor en el map, haciendola corta un "map" es una lookup-table.
// Rust ofrece dos tipos de "maps" `HashMap<K, V>` y `BTreeMap<K, V>`. Los dos comparten muchos
// metodos la diferencia principal es como mantienen a las entradas almacenadas para una rapida
// respuesta a una "query"
// `HashMap` guarda las keys y los valores en una "hash-table", entonces requiere que el type de
// `K` implemente `Hash` y `Eq` que son los requisitos para "hashear"
// Una `BTreeMap` guarda sus entradas en nodos, muchos de los nodos en un `BTreeMap` contienen solo
// pares de "key-value"
// Existen varios metodos para crear un map:
// - `HashMap::new()` y `BTreeMap::new()`: crean nuevos maps
// - `iter.collect()`: Puede usarse para crear y llenar un nuevo `HashMap` o `BTreeMap` desde
//  pares de "key-values" iter debe ser del type: `Iterator<Item=(K, V)>`
// - `HashMap::with_capacity(n)`: crea un `HashMap` nuevo vacio con capacidad para `n` elementos
// Los `HashMaps` y los `BTreeMaps` tienen los mismos metodos para trabajar con "keys" y con
// "values"
// - `map.len()`: nos da la el numero de elementos
// - `map.is_empty`: retorna true si el map no tiene entradas
// - `map.contains_key(&key)`: retorna true si el map no tiene una entrada para el "key" dado
// - `map.get(&key)`: busca en un map por una entrada, si se encuentra dicha entrada retorna
// `Some(r)` donde `r` es una referencia a el valor correspondiente de otra manera retorna `None`
// - `map.get_mut(&key)`: es similar pero retorna una referencia mutable a el valor
// - `map.insert(key, value)`: Insertar una entrada (key, value) dentro del map. Si la entrada ya
// existe solo inserta el valor sobreescribiendo el viejo. El type de retorno es un `Option<V>`
// - `map.extend(iterable)`: itera sobre los items `(K, V)` de un iterable y los inserta en el map
// - `map.append(&mut map2)`: mueve todos los elementos desde `map2` a `map`. Despues de la
// operacion `map2` se vacia
// - `map.remove(&key)`: busca y remueve cualquier entrada que tenga la `key` dada, retorna el
// elemento removido si es que lo encontro. Por ello el type de retorno es `Option<V>`
// - `map.clear()`: remueve todos las entradas
//
// Podemos tambien utilizar la notacion de index o sea `map[&key]`, osea que `map` impl `Index`.
// Sin embargo paniquea si no existe una entrada para el dado "key"(que lo hace mas peligroso que
// un `Option`)
//
// Ya que `BTreeMap<K, V>` mantiene a las entradas ordenadas por "key" esta soporta una operacion
// adicional:
//
// - `btree_map.split_at(&key)`: Divide al `BTreeMap` en dos. Las entradas con "keys" que son
// menores que `key` se dejan en el mismo btree. Retorna un nuevo `BTreeMap<K, V>` conteniendo los
// otras entradas
//
// Entradas: Los dos `HashMap` y `BTreeMap` tienen su correspondiente `Entry` type. El punto es
// elminar los lookups redundantes, por ejemplo el siguiente es el codigo para crear un record de
// un estudiante:
//
if !student_map.contains_key(name) {
    // No; create on
    student_map.insert(name.to_string(), Student::new());
}
// ahora un record existe
let record = student_map.get_mut(name).unwrap();

// esto funciona bien pero accede al `student_map` dos o tres veces haciendo el mismo "lookup" cada
// vez. La idea con las "entries" es que podemos hacer el "lookup" solo una vez, produciendo un
// valor `Entry` que es entonces usado por todos las operaciones siguientes. La siguiente linea es
// equivalente a todo el codigo anterior, exepto que solo hace el "lookup" una vez:
//
let record = student_map.entry(name.to_string()).or_insert_with(Student::new);
// Aqui el valor `Entry` retornado por `student_map.entry(name.to_string())` actua como una
// referencia mutable al lugar donde el map que esta o ocupado por un par "key-value"
// Todas los valores `Entry` son creados de la misma manera:
// `map.entry(key)`: Retorna una `Entry` para la "key" dada. Si no esta en el map retorna un una
// `Entry` que esta vacante
//
// valores `Entry` proveen dos metodos para rellenar entradas vacantes
// - `map.entry(key).or_insert(value)`: asegura que el mapa contiene una entrada con el key dado,
// insertando una nueva entrada con el valor por default si lo necesita. Retorna una referencia
// mutable al nuevo valor. Supongamos que tenemos que contar votos. Podemos escribir lo siguiente:
let mut vote_count: HashMap<String, usize> = HashMap::new();

for name in ballots {
    let count = vote_count.entry(name).or_insert(0);
    *count += 1;
}

// - `map.entry(key).or_insert_with(default_fn)`: Es lo mismo exepto que si necesita crear un
// nueva entrada, llama a la funcion `default_fn()` para producir el nuevo valor, si la entrada ya
// esta en el map entonces no se llama a la funcion. Supongamos que queremos saber que palabra
// aparece en que file, podemos escribir:
//
let mut word_occurrences: HashMap<String, HashSet<String>> = HashMap::new();

for file in file {
    for word in read_words(file)? {
        let set = word_occurrences.entry(word).or_insert_with(HashSet::new);
        set.insert(file.clone());
    }
}

// Iteracion de los maps:
// Hay muchas maneras de iterar sobre un map:
// - Iterar por valor (`for (k, v) in map`) produce pares `(K, V)` esto consume el map
// - Iterar sobre una referencia compartida (`for (k, v) in &map`) produce pares `(&K, &V)`
// - Iterar sobre una referencia mutable (`for (k, v) in &mut map`) produce pares `(&K, &mut V)`.
// Nuevamente no hay manera de tener acceso mutable a una key guardada en un map, porque las
// entradas son organizadas en base a ellas. Como los vectores, maps tienen los metodos`iter()`
// y `iter_mut` los cuales retornan iteradores por referencia ademas tenemos disponibles:
//
// - `map.keys()`: retorna un iterador sobre solo los keys, por referencia
// - `map.values()`: retorna un iterador sobre los valores, por referencia
// - `map.values_mut()`: retorna un iterador sobre los valores, por una referencia mutable
//
// `HashSet<T>` y `BTreeSet<T>`: estos conjuntos de datos agrupados para hacer comprobaciones
// rapidas
//
let b1 = large_vector.contains("needle"); // leeento porque chequea cada elemento
let b2 = large_hash_set.contains("needle"); // raaapido porque tiene un hash lookup

// Un set nunca tiene muchas copias del mismo valor, los mapas y los sets tienen diferentes
// metodos, pero detras de escena un set es un map que solo tiene keys, el lugar de pares de
// "keys-values", de hecho estos dos types de sets son implementados como wrappers pequenios
// alrededor de `HashMap<T, ()>` y `BTreeMap<T, ()>`
//
// - `HashSet::new()` y `BTreeSet::new()` crean sets nuevos
// - `iter.collect()` puede usarse para crear un nuevo set desde un iterador. Si iter produce
// cualquier valor, los duplicados se tiran
// - `HashSet::with_capacity(n)` crea un `HashSet` vacio con espacio para al menos `n` valores
// `HashSet<T>` y `BTreeSet<T>` tienen los mismos metodos basicos en comun:
//
// - `set.len()`: retorna el numero de valores en un set
// - `set.is_empty()`: retorna `true` si el set esta vacio
// - `set.contains(&value)`: retorna `true` si el set contiene el valor dado
// - `set.insert(value)`: agrega el valor al set. Retorna `true` si el valor fue agregado, falso si
// ya es miembro del set
// - `set.remove(&value)`: remueve un valor de el set. Retorna `true` si el valor fue removido,
// `false` si no era miembro del set
//
//
//
// // NOTE(elsuizo:2020-05-21): falta lo de HASH

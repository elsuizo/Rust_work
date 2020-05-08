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
// Un ejemplo piola es el de la libreria serde que nos sirve para serializar datos, osea que
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
let str1 = "piola".to_string();
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
//-------------------------------------------------------------------------
//                        Clousures
//-------------------------------------------------------------------------

//-------------------------------------------------------------------------
//                        enums!!!
//-------------------------------------------------------------------------
// c-style enums
// Create a type Ordering con tres posibles valores llamado variantes o constructores
// enum Ordering {
//     Less,
//     Equal,
//     Greather
// }

// como ya esta en la libreria estandar la podemos importar
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

fn rough_time_to_english(rt: RoughTime) -> String {
    match rt {
        RoughTime::InThePast(units, count)  => format!("{}{}ago", count, units.plural),
        RoughTime::JustNow                  => format!("Just Now!!!"),
        RoughTime::InTheFuture(units, count)=> format!("{}{}for now", count, units.plural)
    }
}

// Supongamos que implementamos un juego de mesa que tiene espacios hexagonales y el jugador
// clikea a donde quiere moverse
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
    Point{x: 0, y: height}=> println!("straight up {} meters"),
    Point{x: x, y: y}     => println!("at ({}m, {}m)", x, y)
}

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

// los types de la libreria estandar que implementan Write son por ejemplo TcpStream,
// File y Vec<u8>. Todos estos types proveen los metodos llamados .write() .flush() ...
// y el codigo que utiliza el Trait Write sin importar el type se ve como:
//
use std::io::Write;

fn say_hello(out: &mut Write) -> std::io::Result<()> {
    out.write("hello world\n")?;
    out.flush()
}

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

// la parte <T: Ord> significa que la funcion puede ser utilizada con argumentos de cualquier
// type T que implementen el Trait Ord
// Los Traits representan un capacidad que un type pude hacer, por ejemplo:
//  - Un type que implementa std::io::Write puede escribir bytes como salida
//  - Un valor que implementan std::iter::Iterator puede producir una secuencia de valores
//  - Un valor que implementa std::clone::Clone puede hacer clones de si mismo en memoria
//  - Un valor que implementa std::fmt::Debug puede ser mostrado en pantalla usando `println!()`
//  con {:?}

// Cuando los metodos de un trait tenemos que importar el Trait en si explicitamente, por ejemplo:

use std::io::Write;

let mut buf: Vec<u8> = vec![];
buf.write_all(b"hello")?;

// Trait objects
// Hay dos maneras de usar traits para escribir codigo polimorfico en Rust: Trait objects y
// generics
// Rust no permite variables del type Write(un Trait) ya que el size de la variantes se debe saber
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
// Pero esto es un poco severo diseño ya que cada nueva ensalada consiste enteramente de un solo
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

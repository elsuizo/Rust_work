# El patron de disenio _Builder_

Cuando tenemos _types_ que tienen muchos parametros y que algunos son opcionales

Si `T` es un _type_ que tiene esas caracteristicas, deberiamos considerar introducir
un nuevo _type_ que sea el encargado de construirlo

 1. Generamos un nuevo _type_ `TBuilder` para configurar incrementalmente a `T`
 2. El constructor del `TBuilder` debera tomar como parametro solo la data que
    se necesita para construir a `T`
 3. El constructor debe ofrecer una serie de metodos convenientes para configurar
    incluyendo metodos para setear entradas compuestas como lo son las _slices_
    incrementalmente. Estos metodos deben devolver `self` para permitir que sean
    concatenados.
 4. El `TBuilder` debe proveer uno o mas metodos "finalizadores" que sean los que
    devuelven el _type_ `T`

El patron _Builder_ es especialmente util cuando construimos un _type_ `T` que tiene
efectos secundarios, como _spawninig_ un thread o lanzar un poceso.

Tenemos dos maneras diferentes de tratar con la propiedad de los _types_

## No consumiendo el `TBuilder`

En algunos casos, construir el _type_ final no requieres que el _builder_ sea
consumido. Por ejemplo el siguiente variante del `Command`:

```rust
// NOTE: el command del std no usa Strings
// esto es una version simplificada

pub Struct Command {
   program: String,
   args: Vec<String>,
   cwd: Option<String>,
   // ... etc
}

impl Command {
   pub fn new(program: String) -> Command {
      Command {
         program,
         args: Vec::new(),
         cwd: None,
      }
   }
   // add an argument to pass to the program
   pub fn arg<'a>(&'a mut self, arg: String) -> &'a mut Self {
      self.args.push(arg);
      self
   }

   // add multiple arguments to pass to the program
   pub fn cwd<'a>(&'a mut self, dir: String) -> &'a mut Self {
      self.cwd = Some(dir);
      self
   }

   // executes th command as a child process, which is returned
   pub fn spawn(&self) -> IoResult<Process> {
      //...
   }
}
```

Notemos que el metodo `spawn`, el cual usa la configuracion del _builder_ para
_spawmear_ un proceso, toma el _builder_ como una referencia inmutable. Esto es
posible porque _spawmear_ un proceso no requiere la propiedad de los datos de
configuracion. El beneficio:

Mediante el uso de prestamos `Command` puede ser utilizado convenientemente para
ambos, una linea o mas complejas construcciones:

```rust
// una linea
Command::new("/bin/cat").arg("file.txt").spawn();

// configuracion compleja
let mut cmd = Command::new("/bin/ls");
cmd.args(".");

if size_sorted {
   cmd.arg("-s")
}

cmd.spawn();
```

## Consumiendo _Builders_

A veces los _builders_ tienen que transferir la propiedad cuando construyen el
type final `T` osea que el metodo que termina debe tomar `self` en lugar de `&self`

```rust
// este es un caso simplificado de `std::thread::Builder`
impl ThreadBuilder {
   /// nombre del thread que vamos a construir, este se usa solo para identificarlo
   /// cuando algo anduvo mal
   pub fn named(mut self, name: String) -> ThreadBuilder {
      self.name = Some(name);
      self
   }

   /// redirecccionamos el thread local a la stdout
   pub fn stdout(mut self, stdout: Box<Writer + Send>) -> ThreadBuilder {
      self.stdout = Some(stdout);
      // ^----> esto tiene propiedad y no puede ser clonado/re usado
      self
   }

   /// Creates and executes a new child thread
   pub fn spawn(self, f: proc():Send) {
      // aca estamos consumiendonos a nosotros mismos
   }
}
```

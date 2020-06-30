# Unsafe code

Todo lo que vimos hasta ahora asegura que el codigo que escribimos sea seguro,
libre de errores de memoria y sin "data-races" de manera automatica gracias a los
types, lifetimes, chequeo de limites y muchas cosas mas que nos ofrece Rust. Pero
esta suerte de pensamiento automatico tiene sus limites, existen muchas tecnicas
de programacion que quedan afuera que Rust no reconoce como seguras. Por eso si
queremos "romper" con esas reglas de manera selectiva Rust nos lo permite. Codigo
"unsafe" es como decirle al compilador; "en este caso, confia en mi". Cuando
hacemos un bloque de codigo `unsafe` podemos llamar a funciones de `C` o `C++`,
dereferenciar punteros llamar a funciones que tambien. Gracias a esta feature
la libreria estandar maneja eficientemente la memoria en los `Vec<T>`, el modulo
de la libreria estandar `std::io` para poder hablar con el sistema operativo.
Las caracteristicas principales de codigo "unsafe" en Rust son:

 - Los bloques de codigo `unsafe` son un limite entre el codigo ordinario de Rust
   y codigo que tiene caracteristicas de ser no seguro

 - Podemos marcar funciones que sean `unsafe`, para alertar a los que la llaman
   de la presencia de posible situaciones que deriven en UB("undefined behavior")

 - Los punteros pelados y sus metodos permiten un acceso a memoria sin restricciones
   y nos dejan construir estructura de datos que de otra manera seria imposible
   por el compilador

 - Entender la definicion de UB nos hace apreciar como puede tener consecuencias
   muy graves mas que obtener resultados que no son los correctos


## "unsafe" de que???


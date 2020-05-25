// NOTE(elsuizo:2020-05-12):
// El iterador `take_while`
// aplica el predicado a cada item y retorna `None` en lugar donde los items producen un `false` al
// predicado del clousure y para cada subsecuente llama a el proximo. Por ejemplo, dado un mail con
// una linea blanca separando el header del body del mensaje, podemos utilizar `take_while` para
// iterar solo en los headers:
fn main() {
    let message = "To: jimb\r\n\
                From: superego<editor@oreily.com>\r\n\
                \r\n\
                Did you get any writing done today???\r\n\
                When will you stop wasting time plotting fractals???\r\n";

    // iteramos solo sobre los headers!!!
    for header in message.lines().take_while(|line| !line.is_empty()) {
        println!("header: {:}", header);
    }
    println!("------------------bodys---------------");
    // NOTE(elsuizo:2020-05-12): le ponemos un skip(1) para que saltee la linea en blanco
    for body in message.lines().skip_while(|line| !line.is_empty()).skip(1) {
        println!("body: {:}", body);
    }
}

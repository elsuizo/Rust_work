# Resumen capitulo 20 del libro nuevo(Programacion asincronica)

Supongamos que estamos haciendo un server de chat. Para cada coneccion hay paquetes
entran que tenemos que parsear, paquetes que salen que tenemos que ensamblar,
parametros de seguridad que debemos manejar, grupos de personas que seguir y mas
cosas. Manejar todos esto para muchas conecciones en simultaneo debe tomar
una organizacion importante.
Idealmente, podemos comenzar con un thread separado por cada coneccion que entra

```rust
use std::{net, threa};
let listener = net::TcpListener::bin(address)?;

for socket_resul in listener.incoming() {
   let socket = socket_result?;
   let groups = chat_group_table.clone();
   thread::spawn(|| {
      log_error(server(socket, groups));
   });
}
```


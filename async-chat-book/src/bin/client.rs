// aca es cuando importamos la libreria que tenemos en `src/lib.rs`
use async_chat_book::utils::{self, ChatResult};
use async_std::io;
use async_std::net;
use async_std::prelude::*;
use std::sync::Arc;

// NOTE(elsuizo:2021-11-12): capaz que es mejor hacer los imports al lado de cada funcion que los
// utiliza, porque asi queda mas claro y no todo arriba
use async_chat_book::FromClient;
/// Funcion para parsear la entrada del usuario
fn parse_command(input: &str) -> Option<FromClient> {
    let (command, rest) = get_next_token(input)?;
    if command == "post" {
        let (group, rest) = get_next_token(rest)?;
        let message = rest.trim_start().to_string();
        return Some(FromClient::Post {
            group_name: Arc::new(group.to_string()),
            message: Arc::new(message),
        });
    } else if command == "join" {
        let (group, rest) = get_next_token(rest)?;
        if !rest.trim_start().is_empty() {
            return None;
        }
        return Some(FromClient::Join {
            group_name: Arc::new(group.to_string()),
        });
    } else {
        eprintln!("Unrecognized command: {:?}", input);
        return None;
    }
}

/// Dado un string como input retornamos un `Some((token, rest))` donde token es la primer palabra
/// sin contar los espacios y rest es el resto del string
fn get_next_token(mut input: &str) -> Option<(&str, &str)> {
    input = input.trim_start();
    if input.is_empty() {
        return None;
    }
    match input.find(char::is_whitespace) {
        Some(space) => Some((&input[0..space], &input[space..])),
        None => Some((input, "")),
    }
}

async fn send_commands(mut to_server: net::TcpStream) -> ChatResult<()> {
    println!(
        "Commands: \n\
             join GROUP\n\
             post GROUP MESSAGE...\n\
             Type Control-D(on UNIX) or Control-Z(on Windows)\
             to close connection"
    );

    let mut command_lines = io::BufReader::new(io::stdin()).lines();
    while let Some(command_result) = command_lines.next().await {
        let command = command_result?;
        let request = match parse_command(&command) {
            Some(request) => request,
            None => continue,
        };
        utils::send_as_json(&mut to_server, &request).await?;
        to_server.flush().await?;
    }
    Ok(())
}

use async_chat_book::FromServer;

async fn handle_replies(from_server: net::TcpStream) -> ChatResult<()> {
    // aca leemos lo que nos trajo la conexion
    let buffered = io::BufReader::new(from_server);
    // aca lo convertimos a json
    let mut reply_stream = utils::receive_as_json(buffered);
    // aca es cuando usamos la magia de los Streams(que son como iterators pero asincronicos)
    // capaz que en proximas versiones de Rust podamos hacer un simple for aca...
    while let Some(reply) = reply_stream.next().await {
        match reply? {
            FromServer::Message {
                group_name,
                message,
            } => {
                println!("message posted to: {}: {}", group_name, message);
            }
            FromServer::Error(message) => {
                println!("error from server: {}", message)
            }
        }
    }
    Ok(())
}

//-------------------------------------------------------------------------
//                        main function
//-------------------------------------------------------------------------
use async_std::task;

fn main() -> ChatResult<()> {
    let address = std::env::args().nth(1).expect("Usage: client ADDRESS:PORT");

    task::block_on(async {
        let socket = net::TcpStream::connect(address).await?;
        socket.set_nodelay(true)?;

        let to_server = send_commands(socket.clone());
        let from_server = handle_replies(socket);
        from_server.race(to_server).await?;

        Ok(())
    })
}

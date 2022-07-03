//! tests/healt_check.rs
//!
//! `tokio::test` es el equivalente de testeo para el tokio::main

// use zero2prod::run;
use actix_web::rt::net::TcpListener;

#[tokio::test]
async fn healt_check_works() {
    let address = spawn_app();
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/healt_check", &address))
        .send()
        .await
        .expect("Failed to executed");

    // assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// lanzamos la aplicacion en el backgroud de alguna manera
//
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

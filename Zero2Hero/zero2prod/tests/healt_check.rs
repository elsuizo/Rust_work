//! tests/healt_check.rs
//!
//! `tokio::test` es el equivalente de testeo para el tokio::main

// use zero2prod::run;

#[tokio::test]
async fn healt_check_works() {
    spawn_app();
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
fn spawn_app() {
    let server = zero2prod::run("127.0.0.1:0").expect("Failed to bin address");
    // lanzamos el server como un proceso en el backgroud
    // tokio::spawn retorna un handle para spamear un Future
    // pero aca no lo usamos por ahora...
    let _ = tokio::spawn(server);
}

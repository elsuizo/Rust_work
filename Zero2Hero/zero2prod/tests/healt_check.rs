//! tests/healt_check.rs
//!
//! `tokio::test` es el equivalente de testeo para el tokio::main

use std::net::TcpListener;

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

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    // act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let tests_cases = vec![
        ("name=le%20guin", "missing the mail"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in tests_cases {
        // act
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            // mensaje adicional que ponemos para que sea mas claro todo
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        );
    }
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

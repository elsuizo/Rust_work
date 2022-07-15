use std::net::TcpListener;

#[tokio::test]
async fn healt_check_works() {
    let address = spawn_app();
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health_check", &address))
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
        .post(&format!("{}/subcriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bin random port");
    // guardamos el port que nos ha asignado por el Sistema operativo
    let port = listener.local_addr().unwrap().port();
    let server = parsing_from_post_request::run(listener).expect("Failed to bin address");
    // lanzamos el server como un proceso en el backgroud
    // tokio::spawn retorna un handle para spamear un Future
    // pero aca no lo usamos por ahora...
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}

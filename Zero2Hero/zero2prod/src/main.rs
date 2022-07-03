use zero2prod::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // hacemos un buble-up del error si es que hay alguno
    run("127.0.0.1")?.await
}

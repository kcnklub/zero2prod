use std::net::TcpListener;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration =
        zero2prod::configuration::get_configuration().expect("Unable to get configuration");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Unable to bind to socket");
    run(listener)?.await
}

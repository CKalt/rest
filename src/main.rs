use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use std::fs::File;
use daemonize::Daemonize;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let stdout = File::create("daemon.out").unwrap();
    let stderr = File::create("daemon.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file("test.pid")  // Every method except `new` and             `start`
        .chown_pid_file(false) // is optional, see `Daemonize` documentation
        .working_directory("./") // for default behaviour.
        .stdout(stdout)  // Redirect stdout to `/home/chris/projects/rust/daemon/daemon.out`.
        .stderr(stderr)  // Redirect stderr to `/home/chris/projects/rust/daemon/daemon.err`.
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => {
            println!("Success, daemonized");
            HttpServer::new(|| {
                App::new()
                    .service(hello)
                    .service(echo)
                    .route("/hey", web::get().to(manual_hello))
            })
            .bind("127.0.0.1:8080")?
            .run()
            .await?
        },
        Err(e) => eprintln!("Error, {}", e),
    }
    Ok(())
}

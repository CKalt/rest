use std::fs::File;
use daemonize::Daemonize;

use actix_web::{web, App, HttpServer};
use std::sync::Mutex;

struct AppStateWithCounter {
    counter: Mutex<i32>, // <- Mutex is necessary to mutate safely across threads
}

async fn index(data: web::Data<AppStateWithCounter>) -> String {
    let mut counter = data.counter.lock().unwrap(); // <- get counter's MutexGuard
    *counter += 1; // <- access counter inside MutexGuard

    format!("Request number: {}", counter) // <- response with count
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

            let counter = web::Data::new(AppStateWithCounter {
                counter: Mutex::new(0),
            });

            HttpServer::new(move || {
                   // move counter into the closure
                   App::new()
                       // Note: using app_data instead of data
                       .app_data(counter.clone()) // <- register the created data
                       .route("/", web::get().to(index))
               })
               .bind("127.0.0.1:8080")?
               .run()
               .await?
        },
        Err(e) => eprintln!("Error, {}", e),
    }
    Ok(())
}

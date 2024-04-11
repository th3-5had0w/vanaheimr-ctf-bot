use std::process::exit;

mod arena_generator;
mod core_api;
mod telegram_bot;
mod network;
mod auth;


fn setup() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(
        move |info| {
            original_hook(info);
            exit(1);
        }
    ));
}

#[tokio::main]
async fn main () {
    //setup();
    arena_generator::main_arena_generator().await;
    //network::intercepting_arena().await;
    auth::read_db().await;
    telegram_bot::main_telegram_bot().await;
    //core_api::actix_web_main();
    /*
    let mut cmdline_args: Vec<String> = std::env::args().collect();
    if cmdline_args.len() != 3 {
        println!("Usage: {} <ip> <port> <id_number>", cmdline_args.first().unwrap());
        exit(0);
    }

    cmdline_args.reverse();
    cmdline_args.pop();
    let join_handle = std::thread::spawn(rocket_main);
    let client_join_handle = std::thread::spawn(move || 
        client_main(
            cmdline_args.pop().unwrap().as_str(),
            cmdline_args.pop().unwrap().as_str(),
            cmdline_args.pop().unwrap().as_str()
        )
    );
    */

    //let _ = client_join_handle.join();
    //let join_handle = std::thread::spawn(rocket_main);
    //let _ = join_handle.join();
}
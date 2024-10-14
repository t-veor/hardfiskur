mod uci;

use hardfiskur_engine::Engine;
use uci::main_loop;

// Ensures a panic from the background thread exits the engine, rather than just
// leaving the stdin reading thread waiting forever for a response.
fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        original_hook(panic_info);
        std::process::exit(1);
    }));
}

fn main() {
    install_panic_hook();

    let args: Vec<_> = std::env::args().collect();
    let mut engine = Engine::new();

    if args.len() == 2 && args[1] == "bench" {
        let (nodes, time) = engine.bench();
        let nps = nodes * 1000 / time.as_millis() as u64;
        println!("nodes {nodes} time {} nps {nps}", time.as_millis());
        return;
    }

    main_loop(&mut engine)
}

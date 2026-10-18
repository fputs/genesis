fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "\
Genesis

  cargo run -p game
  cargo run -p game -- --headless
  cargo run -p game -- --help

Explore: D walks east toward the guard, A west, W north, S south. 1–4 selects a portrait.
Combat: M then click a lit square to Stride. T then click a lit square to Step. F then click the guard to Strike. R raises a shield. Space ends the turn. When Shield Block is legal, B uses it and N does not.
Without a display, the process runs the headless smoke and exits."
        );
        return;
    }

    let requested = args.iter().any(|arg| arg == "--headless");
    let display =
        std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some();
    if requested || !display {
        if !display && !requested {
            eprintln!("No display detected; running the headless smoke instead.");
        }
        match game::headless_report() {
            Ok(report) => println!("{report}"),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
        return;
    }

    game::run_window();
}

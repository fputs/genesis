fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "\
Genesis Editor

  cargo run -p editor
  cargo run -p editor -- fixtures/rooms/small_room.json
  cargo run -p editor -- --help

Load reads a room file. Choose Stone, Grass, or Dirt, then click a square.
Save writes that same file. Esc closes the window."
        );
        return;
    }

    let mut path = None;
    for arg in &args {
        if arg.starts_with('-') {
            eprintln!("unknown argument {arg}");
            eprintln!("run with --help");
            std::process::exit(1);
        }
        if path.is_some() {
            eprintln!("expected one room file");
            std::process::exit(1);
        }
        path = Some(std::path::PathBuf::from(arg));
    }
    let path = path.unwrap_or_else(|| std::path::PathBuf::from("fixtures/rooms/small_room.json"));

    let display =
        std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some();
    if !display {
        eprintln!("Genesis Editor needs a display to open the map window.");
        std::process::exit(1);
    }

    editor::run(path);
}

# terminal-ui

## Run
```
cargo run -p terminal-ui
```

## Structure

```
src/
├── event.rs   -> handles the terminal events (receive summary from server, and key press)
├── lib.rs     -> module definitions
├── main.rs    -> initializes/exits the terminal interface, handles the key press events and updates the application.
└── ui.rs      -> renders the widgets / UI
```

### ratatui

> The library is based on the principle of immediate rendering with intermediate buffers. This means that at each new frame you should build all widgets that are supposed to be part of the UI. While providing a great flexibility for rich and interactive UI, this may introduce overhead for highly dynamic content. So, the implementation try to minimize the number of ansi escapes sequences generated to draw the updated UI. In practice, given the speed of Rust the overhead rather comes from the terminal emulator than the library itself.

### crossterm

> Crossterm is a pure-rust, terminal manipulation library that makes it possible to write cross-platform text-based interfaces.
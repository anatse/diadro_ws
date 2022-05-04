# Application to test one code for WASM and Desktop drawing in couple with WebSocket communications
For UI and drawing using [egui](https://github.com/emilk/egui) library

* [dserver](./dserver/) - WebSocket server
* [diadro](./diadro/) - Desktop application or WASM library, depending on selected compile target

## Code view
If you're using Visual Studio Code use plugin [Better Comments](https://marketplace.visualstudio.com/items?itemName=aaron-bond.better-comments) for a better perception.

## Compilation and start

### Compile WASM

* Build WASM file:
```bash
    cd ./diadro
    ./build-web.sh
```

* WASM file and other files needed to run HTTP page located in [diadro/docs](./diadro/docs/)

### Compile and run server
Three are two methods to run 

* Through the IDE (Vscode od Atom or Intellij Idea) run main function from [dserver/src/main.rs](./dserver/src/main.rs)
* Through the command line using cargo
```bash
cargo clean
cargo run --package dserver
```
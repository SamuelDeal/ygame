# YGame

First rust project, implementing a simple board game

## Usage

Be sure to have the wasm binary built (see Building section).
Then you can run `cargo run` or `ygame -p MY_PORT`
Finally serve static files, using nginx or whatever (see `misc/ygame.nginx.conf.template.conf`)

## Building

You should first compile the wasm binary. 
You will need to install:
* [the rust toolchain](https://www.rust-lang.org/tools/install)
* [cargo-make](https://github.com/sagiegurari/cargo-make): `cargo install cargo-make` 
(it requires build-essentials, pkg-config and libssl-dev on linux)

And then run:
```
cargo make build
```

## Deployment

Run `cargo make build`
Then you only need to deploy the `ygame` executable located in the `target` folder for server-side
And the files located in the ``server/static`` folder

Now serve files using a reverse proxy. An example of nginx conf is located in the `misc` folder

## Connection and port

Default port for websocket is 8000 server-side. You can specify a diffrent one is `ygame -p 80` or defining the `YGAME_PORT` env variable
By default the server is listening to 127.0.0.1 but you can change it using `ygame -l 0.0.0.0`  or defining the `YGAME_LISTEN` env variable

The client will by default connect to the same server address, using https only if the page use it.
You can specify a different hostname or port in the file `server/static/client.js`



## See also

### libs and tools:

* [wasm-bindgen](https://rustwasm.github.io/docs/wasm-bindgen/)
* [cargo-make](https://github.com/sagiegurari/cargo-make)
* [serde](https://serde.rs/)
* [clap](https://github.com/clap-rs/clap)
* https://docs.rs/futures/0.3.16/futures/channel/mpsc/fn.channel.html
* https://docs.rs/wasm-bindgen-futures/0.4.25/wasm_bindgen_futures/fn.spawn_local.html
* https://github.com/RazrFalcon/cargo-bloat

### Usefull tutorials:

* https://www.chinedufn.com/3d-webgl-basic-water-tutorial/
* https://levelup.gitconnected.com/websockets-in-actix-web-full-tutorial-websockets-actors-f7f9484f5086
* https://rustwasm.github.io/book/game-of-life/introduction.html
* https://rustwasm.github.io/docs/book/introduction.html
* http://www.sheshbabu.com/posts/rust-wasm-yew-single-page-application/


### Too lately discovered projects

* https://github.com/not-fl3/macroquad
* https://trunkrs.dev/configuration/
* https://docs.rs/gumdrop/0.8.0/gumdrop/
* https://crates.io/crates/stdweb
* yew

### Other stolen things:

* https://normform.netlify.app/
* https://vanillacss.com/

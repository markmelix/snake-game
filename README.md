# Snake Game Kit
This project provides snake game server and gui client. Thus, to play you have
to start the server and connect using a client.

Note that you aren't force to use exactly client provided by this
repository. You can use any client which is supported by the server or even
write your own (and not even necessarily on Rust!) one!

Note that now game is in the alpha stage, so it won't be published and this
repository becomes public only after everything in todo lists below will be
completed and we will be completely sure nothing besides little bugs and some
major updates is being changed in the future. Now project is very messy and it
should be refactored and updated a lot.

## TODO list
### Server backlog
- [x] Generate random coordinates of a new snake depending on all parts
- [x] Fix snake change direction algorithm
- [x] Document specifications for clients
- [x] Refactor to make the library more flexible (create traits)
- [x] Implement apples
- [x] Add more abilities to setup server. For instance, add ability to control
	  what color and length will snakes have after being spawned
- [x] Implement system to protect snake from death when it's going to turn 180
      degrees
- [ ] Implement server console with admin features
- [ ] Create library bindings for Python
- [ ] Optimize algorithms and make server more fast
- [ ] Get rid of most `unwrap`s and replace them with error handling

### Logger backlog
- [x] Implement logger library which will contain everything other crates in
	  this project may need for logging.

### Official client backlog
- [ ] Get rid of most `unwrap`s and replace them with the error handling
- [ ] Make GUI more beautiful
- [ ] Add some information into GUI to make client more informative
- [ ] Make game more smooth
- [ ] Look for more elegant and comfortable GUI library and move in it

## How to play
After you clone this repository, launch the server:
```bash
cargo run --bin server
# for detailed information/configuration: cargo run --bin server -- --help
```

Next, you should connect using a client. If you use the official one, type this
into another terminal from the repository root to launch the client:
```bash
cargo run --bin client
# for detailed information/configuration: cargo run --bin client -- --help
```
Now fill necessary fields and connect to the started server. Enjoy the game!

## How to write own client
If you want to write your own client which will be supported by server, you have
to choose in what language you will write it.

If you use Rust, then there're great news because you can use a standart library
provided by the project to make your client writing hundred times easier.

If you use Python, than you can wait until I develop library bindings for it (it
won't be soon) and develop your client hundred times easier using these
bindings.

If you don't want to wait or you don't write on Python, than you should
implement everything on your own.

For detailed instructions on how to implement your own client, see game
library documentation.

## FAQ
### Where is game library documentation?
Run one of these commands in the project root to open generated documentation in
your default browser (--no-deps flag forces generator not to build dependencies
docs):
```bash
cargo doc --package server --no-deps --open # open server docs
cargo doc --package game --no-deps --open # open game abstractions docs
cargo doc --package client --no-deps --open # open official client docs
cargo doc --package logger --no-deps --open # open logger docs
```

### How do I get my public ip?
Run this command in your terminal to get your public ip:
```bash
curl ifconfig.me
```

### How do I play with friends?
To play together you should launch the server and get your public ip. You also
must have ports on which the server is started on opened. After that, every
player should run a client and connect to the server using your public ip and
the port separated by a colon.

## Contribution
Contributions are very welcome!

## License
Snake Game Kit is licensed under [MIT](LICENSE) license.

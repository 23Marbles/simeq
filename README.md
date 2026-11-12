# simeq

`simeq` solves systems of linear equations using exact rational arithmetic. It provides both a command-line program and a Rust library.

## Command line

Install the CLI:

```sh
cargo install simeq
```

Run it and enter one equation per line. End input with a blank line or EOF:

```text
$ simeq
x + y = 3
x - y = 1

```

The program displays the linear system, its row echelon forms, and the solution.

## Library

Add `simeq` to a Rust project:

```sh
cargo add simeq
```

Parse and solve a system:

```rust
use simeq::solver::LinearSystem;

fn main() {
    let mut system = "x + y = 3\nx - y = 1/4"
        .parse::<LinearSystem>()
        .expect("valid linear system");

    // Unique Solution
    // x = 13/8
    // y = 11/8
    println!("{}", system.solve());
}
```

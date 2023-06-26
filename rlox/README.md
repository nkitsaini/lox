# Writing tests
This project uses snapshot testing using `insta` crate.
Install the tool with: 
```sh
cargo install cargo-insta
```

To update snapshot for a test run:
```sh
cargo insta review
```

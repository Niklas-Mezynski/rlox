# A rust based interpreter for the Lox language

This is my attempt of creating a rust based interpreter for Lox. A language created for learning everything about compilers/interpreters/VMs - whatever you want to call it. Everything in here is based on the wonderful book [Crafting Interpreters](https://craftinginterpreters.com/contents.html) by Robert Nystrom.

In Chapter II of that book, he creates this exact same Lox interpreter in Java and I thought it would be a fun challenge to do so in Rust. I am still learning rust and still have to optimized some things here and there... but ey, it works!

## Instruments

First build the binary (either debug or release)

```bash
cargo build
```

Then sign the App and use it in Instruments.app

```bash
codesign --force --sign - --entitlements debug.entitlements --timestamp=none ./target/debug/rlox
```

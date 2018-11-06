[![Build Status](https://travis-ci.org/paholg/rustybar.svg?branch=master)](https://travis-ci.org/paholg/rustybar)

Rustybar
=====

A simple, customizable statusbar written in Rust. It is designed to work with XMonad,
but should work with any similar window manager. Its purpose is to be lightweight but
featureful.

It currently uses dzen2 as a backend, but may eventually be changed to do everything
internally for increased features.

The included `example_config.toml` will be used if you do not have an existing config. It also gives
you a walkthough on setting it up. The best way to get started with rustybar is to run it,
generating your config file, and then play with the generated file.

To build, simply run
```
Cargo build --release
```

# Known Issues

* Rustybar will automatically resize if your resolution changes. However, it waits for threads to
  join first. The brightness bar blocks until brightness changes. So, on resolution change, rustybar
  will be broken until your brightness changes if using that bar.

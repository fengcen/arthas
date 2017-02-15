
Arthas
======
[![Build Status](https://travis-ci.org/fengcen/arthas.svg?branch=master)](https://travis-ci.org/fengcen/arthas)
[![status](http://www.repostatus.org/badges/latest/wip.svg)](http://www.repostatus.org/#wip)
[![docs](https://docs.rs/arthas/badge.svg?version=0.2.0)](https://docs.rs/arthas)

Arthas is an in-memory structure database.

## [Document](https://docs.rs/arthas)

## Status
Initial development is in progress, but there has not yet been a stable, usable release suitable for the public. Use at your own risk.

## Prerequisites
Arthas required latest **Nightly** Rust.

## Features
* Persistence
* Automatically update fields
* Automatic indexing
* Use structure without ORM
* No complicated setup is required


## Usage
Add dependencies to Cargo.toml

```toml
[dependencies]
arthas = "^0.2"
arthas_plugin = "^0.1"
serde_derive = "^0.9"
```

In your `main.rs` or `lib.rs`:

```rust
#![feature(plugin, custom_derive)]
#![plugin(arthas_plugin)]

#[macro_use]
extern crate serde_derive;
extern crate arthas;
```

## CRUD Methods
- [x] insert()
- [x] remove()
- [x] replace()
- [x] find()
- [x] find_one()
- [x] count()

## Query Methods
- [x] id()
- [x] limit()
- [x] offset()
- [x] field()
- [x] len()
- [x] eq()
- [x] ne()
- [x] gt()
- [x] lt()
- [x] ge()
- [x] le()
- [x] desc()
- [x] asc()

## Examples
Examples can be found in the [Documentation](https://docs.rs/arthas).

## License
arthas is primarily distributed under the terms of the MIT license.
See [LICENSE](LICENSE) for details.

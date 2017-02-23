
Arthas
======
[![Build Status](https://travis-ci.org/fengcen/arthas.svg?branch=master)](https://travis-ci.org/fengcen/arthas)
[![status](http://www.repostatus.org/badges/latest/wip.svg)](http://www.repostatus.org/#wip)
[![docs](https://docs.rs/arthas/badge.svg)](https://docs.rs/arthas)

Arthas is an in-memory structure database.

## [Document](https://docs.rs/arthas)

## Status
Initial development is in progress, but there has not yet been a stable, usable release suitable for the public. Use at your own risk.

## Prerequisites
Arthas required Rust 1.15 or above.

## Upgrade to arthas 0.3 and arthas_derive
1. Rename all your data files to only contain struct name. For example, rename "model.user.User" to "User".
2. Replace attribute `#[arthas]` to `#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Arthas)]`
3. If you use attribute value like `#[arthas(one)]`, replace with the attribute `#[arthas(is_one)]`
4. If you use the rename attribute like `#[arthas_rename("from = to")]`, replace with the attribute `#[arthas(rename = "from = to")]`
5. If you use both `#[arthas(one)]` and `#[arthas_rename("from = to")]`, replace with `#[arthas(is_one, rename = "from = to")]`

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
arthas = "^0.3"
arthas_derive = "^0.1"
serde_derive = "^0.9"
```

In your `main.rs` or `lib.rs`:

```rust
extern crate arthas;
#[macro_use]
extern crate arthas_derive;
#[macro_use]
extern crate serde_derive;
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

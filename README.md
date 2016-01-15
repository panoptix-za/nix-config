nix-netconfig
=============

A simple Rust library that provides an API for manipulating network configurations for varius *nix operating systems.

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
nix-netconfig = "0.1.0"
```

Next add this to your code:

```rust
extern crate nix-netconfig;
```

## Public API

TODO, document the API, for now you'll have to refer to the src code.


## Roadmap 

- [x] Display Static/DHCP/DNS
- [x] Support changing from DHCP to Static (and reverse) for Arch/Systemd
- [ ] Add/remove DNS
- [ ] Handle restart/reconfig of interfaces
- [ ] Ubuntu Upstart support
- [ ] Ubuntu Systemd support

## License

The MIT License (MIT)

Copyright (c) 2016 Panoptix CC.

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

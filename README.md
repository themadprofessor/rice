# Rice

[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)
![GitHub](https://img.shields.io/github/license/themadprofessor/rice?style=flat-square)
[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fthemadprofessor%2Frice.svg?type=shield)](https://app.fossa.com/projects/git%2Bgithub.com%2Fthemadprofessor%2Frice?ref=badge_shield)
[![](https://tokei.rs/b1/github/themadprofessor/rice)](https://github.com/themadprofessor/rice).

A clone of the wonderful [Ananicy](https://github.com/Nefelim4ag/Ananicy) tool to Rust.
`rice` adjusts the nice-value, cgroup, ionice-value, and io-class of running processes according to a set of rules.

## Install
`rice` depends on stable Rust and libc.

```
cargo install
```

By default, `rice` outputs to stderr through the use of the
[pretty_env_logger](https://crates.io/crates/pretty_env_logger/) crate.
`rice` can be configured to output to syslog using the [syslog](https://crates.io/crates/syslog/) crate, by changing the
compile-time features:

```
cargo install --no-default-features --features "syslog"
```

## Usage

```
rice
```

## Configuration
Since this is a clone of [Ananicy](https://github.com/Nefelim4ag/Ananicy), it is configured the same way and currently
assumes [Ananicy](https://github.com/Nefelim4ag/Ananicy) 's rules exist in its config directory.
See [here](https://github.com/Nefelim4ag/Ananicy#configuration) for details on how to configure Ananicy and in turn
`rice`.

## Contributing

PRs accepted.

## License

MIT © Stuart Reilly

[![FOSSA Status](https://app.fossa.com/api/projects/git%2Bgithub.com%2Fthemadprofessor%2Frice.svg?type=large)](https://app.fossa.com/projects/git%2Bgithub.com%2Fthemadprofessor%2Frice?ref=badge_large)
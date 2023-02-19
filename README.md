`goup`: A Go Version Manager
============================

Downloading the official Go binaries and installing them can be pretty annoying.
It's not hard, _per se_, but going to [go.dev](https://go.dev) and downloading
the tarball, then going to the
[install documentation](https://go.dev/doc/install) to find where they told you
to put the files is needlessly complicated, especially when other languages
(like Rust) have utilities for that kind of busywork (like rustup).

What `goup` allows you to do is to simply check for and run updates on your Go
installation from the command line. This makes it easy to add updating Go to an
existing system update routine. Additionally, `goup` can serve as a verison
multiplexer. If an update breaks your build, you can roll back to a previous
version. Given Go's stability, that scenario is unlikely, but `goup` supports it
nonetheless.

## Example Commands

Some example commands to give you an idea of the `goup` "flavor":

```bash
$ goup list # list installed Go versions, as well as those that are available
$ goup update # install and enable the latest version of Go
$ goup install go1.19.4 # install version go1.19.4 (if available on go.dev)
$ goup clean # remove installations that are out of date
$ goup --help # get help and see all commands
```

## Setup

Put the goup executable somewhere on your path. Some reasonable places might be
`~/bin`, `~/.local/bin`, or `~/.goup/bin`. Remember to add the location to your
path, if it is not there by default.

You will need to set your `GOROOT` environment variable to point to the location
where `goup` installs Go. You will also want to add the bin folder for your Go
install to your path so that your shell can find the Go executable. You can do
this by adding the following to your `.bashrc`:

```bash
# ~/.bashrc
export GOROOT="$HOME/.goup/go"
export PATH="$GOROOT/bin"
```

## Limitations

Many of the limitations of `goup` are related to the project scope. This is a
relatively small utility and it does what I need it to do.

- Currently, `goup` is only developed for Linux. Theoretically, it should work
  for Mac and other Unix systems as well. Windows is explicitly not supported.
- We download binaries from [go.dev](https://go.dev/dl), so only the currently
  supported versions are available.
- We cannot build from source, and only have support built for architectures
  like x86, x86_64, and aarch64.
- `goup` will always use the `~/.goup` folder for its files, and does not
  provide a system-wide install.

## Acknowledgements

`goup` is made possible thanks to the generous contributions of others!

| Crate       | Owner / Maintainer                        | License           |
| ----------- | ----------------------------------------- | ----------------- |
| clap        | Kevin K.                                  | MIT or Apache-2.0 |
| console     | Armin Ronacher and Pavan Kumar Sunkara    | MIT               |
| flate2      | Alex Crichton and Josh Triplett           | MIT or Apache-2.0 |
| home        | Brian Anderson                            | MIT or Apache-2.0 |
| indicatif   | Armin Ronacher and Dirkjan Ochtman        | MIT               |
| lazy_static | Marvin Lobel                              | MIT or Apache-2.0 |
| regex       | rust-lang/libs                            | MIT or Apache-2.0 |
| serde       | David Tolnay                              | MIT or Apache-2.0 |
| serde_json  | David Tolnay                              | MIT or Apache-2.0 |
| tar         | Alex Crichton                             | MIT or Apache-2.0 |
| ureq        | Martin Algesten and Jacob Hoffman-Andrews | MIT or Apache-2.0 |

And a special thanks is due to the rustup team for inspiration!

## License

This project is licensed under the Mozilla Public License V2.

Joseph Skubal 2022-2023

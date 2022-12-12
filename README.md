# `goup`: A Go Version Manager

Downloading the official Go binaries and installing them can be pretty annoying.
It's not hard, _per se_, but going to [go.dev](https://go.dev) and downloading
the tarball, then going to the
[install documentation](https://go.dev/doc/install) to find the location they
told you to put the files in is needlessly complicated, especially when other
languages (like Rust) have utilities for that kind of busywork (like rustup).

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
```

## Setup

If you compiled a version of `goup` from source manually or downloaded it, it is
recommended that you save the executable as `~/.goup/bin/goup`. The executable
should work correctly from anywhere on your path, but it will always create and
saved data in the `~/.goup` directory, so you might as well keep the executable
there as well.

You will need to set your `GOROOT` environment variable to point to the location
where `goup` installs Go. You will also want to add the bin folder for your Go
install to your path so that your shell can find the Go executable. You can do
this by adding the following to your `.bashrc`:

```bash
# ~/.bashrc
export GOROOT="$HOME/.goup/go"
export PATH="$GOROOT/bin:$HOME/.goup/bin:$PATH"
```

## Acknowledgements

`goup` is made possible thanks to the generous contributions of others!

| Crate       | Owner / Maintainer                        | License           |
| ----------- | ----------------------------------------- | ----------------- |
| clap        | Kevin K.                                  | MIT or Apache-2.0 |
| directories | soc                                       | MIT or Apache-2.0 |
| flate2      | Alex Crichton and Josh Triplett           | MIT or Apache-2.0 |
| lazy_static | Marvin Lobel                              | MIT or Apache-2.0 |
| regex       | rust-lang/libs                            | MIT or Apache-2.0 |
| serde       | David Tolnay                              | MIT or Apache-2.0 |
| serde_json  | David Tolnay                              | MIT or Apache-2.0 |
| tar         | Alex Crichton                             | MIT or Apache-2.0 |
| ureq        | Martin Algesten and Jacob Hoffman-Andrews | MIT or Apache-2.0 |
| yansi       | Sergio Benitez                            | MIT or Apache-2.0 |

And a special thanks is due to the rustup team for inspiration!

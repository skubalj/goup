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

Setup can be automated by running the `install.sh` script from the root of this repository.

```bash
git clone https://github.com/skubalj/goup.git
cd goup
./install.sh
```

This script will install goup to `$GOPATH/bin`. You can customize the location of GOPATH by
specifying the variable when running this script. As this script can be used to bootstrap a system
with no existing go installation, it will create an environment variable file that can be called 
from your `~/.bashrc` to configure your shell.

## Limitations

Many of the limitations of `goup` are related to the project scope. This is a
relatively small utility and it does what I need it to do.

- Currently, `goup` is only developed for Linux. Theoretically, it should work
  for Mac and other Unix systems as well. Windows is explicitly not supported.
- We download binaries from [go.dev](https://go.dev/dl), so only the currently
  supported versions are available.
- We cannot build go from source, and the only architectures mapped are x86, 
  x86_64, and aarch64.
- `goup` will always use the `$GOPATH/goup` folder for its files, and does not
  provide a system-wide install.

## Acknowledgements

`goup` is made possible thanks to the generous contributions of others!

| Crate       | Owner / Maintainer                        | License           |
| ----------- | ----------------------------------------- | ----------------- |
| anyhow      | David Tolnay                              | MIT or Apache-2.0 |
| clap        | Kevin K.                                  | MIT or Apache-2.0 |
| console     | Armin Ronacher and Pavan Kumar Sunkara    | MIT               |
| flate2      | Alex Crichton and Josh Triplett           | MIT or Apache-2.0 |
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

Joseph Skubal 2022-2025

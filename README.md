Urlnao
======

[![License](http://img.shields.io/badge/license-AGPL-brightgreen.svg?style=flat-square)](LICENSE)

## What is Urlnao?

Urlnao is an upload service for [file sharing with weechat-android](https://github.com/ubergeek42/weechat-android/wiki/File-sharing).

* [What does it do?](#what-does-it-do)
* [Building](#building)
* [How does it work?](#how-does-it-work)
* [Developing](#developing)
* [Changelog](#changelog)
* [License](#license)

## What does it do?

### Urlnao handles uploads, returns URLs and serves the uploaded files:
1. A POST request is issued by a client with some file(s) to share
2. Urlnao saves the uploaded file(s)
3. A publicly reachable URL is then returned for each uploaded file

Example upload request using curl instead of [weechat-android](https://github.com/ubergeek42/weechat-android):
```shell
$ curl --file file=@/path/to/some/video_file.webm https://urlnao.example.com/up
https://urlnao.example.com/f/wLM1
```

The file will then be accessible via the returned URL.
```shell
$ xdg-open https://urlnao.example.com/f/wLM1
```

Example upload request using curl with multiple files:
```shell
$ curl \
    --file file1=@/path/to/some/video_file.webm \
    --file file2=@/path/to/some/image_file.png \
    https://urlnao.example.com/up
https://urlnao.example.com/f/wLM1
https://urlnao.example.com/f/af6
```

## Building

I recommend using Nix to build this package, but you can also just use `cargo build --release`.
```shell
nix build -f . urlnao
```

## How does it work?

### Requests

All incoming requests need to be passed to Urlnao's Unix domain socket,
ideally using a proxy like Nginx for handling TLS termination.

Example request with curl:
```shell
$ curl \
  --file file=@/path/to/some/image_file.png \
  --unix-socket /path/to/urlnao.sock \
  http://localhost/up
https://urlnao.example.com/f/02f6a
```

### Uploads

Files are uploaded by issuing POST requests to the endpoint `/up`.
Every multipart request may contain more than one file,
resulting in one public URL for each uploaded file in the response.

The returned URLs are configurable and end in a randomly generated
short ID, (currently) being constructed of 3 to 8 alphanumeric characters:
Example:
```
https://u.example.com/f/02f6a
```

### Downloads

Accessing the returned URL after a successful upload leads to a redirect,
where the target location contains the original client-supplied filename:

Simplified example:
```
> GET https://u.example.com/f/02f6a
< 301 https://u.example.com/d/image_file.png
> GET https://u.example.com/d/image_file.png
```

### State

A simple list of all current uploads,
including their respective checksum and filename,
is served from the `/state` endpoint.

Example:
```shell
$ xdg-open https://u.example.com/state
```

### Access Control

Access control must be implemented by the upstream proxy,
allowing for any kind of authentication that both the client and proxy support.

Access to the following endpoints should be restricted to authorized users:
* `/up` (endpoint for uploads)
* `/state` (endpoint listing all uploads)

Alternatively only the two endpoints:
* `/f` (default for `--shortid-path`)
* `/d` (default for `--download-path`)

can be publicly exposed, allowing access to everything else only from a trusted network.

## Developing

Development is done with the help of [Nix](https://nixos.org/)

### Development Environment

```shell
$ nix-shell
...
[nix-shell]$ cargo build --release
```

### Integration Test

A simple integration test can be found here: [nix/test.nix](nix/test.nix)

To run the NixOS VM test run:
```shell
$ nix build -f . test
```

To run the test VM interactively run:
```shell
$ nix build -f . text.driver
$ ./result/bin/nixos-test-driver
```

## Changelog

### v0.3.0
**BUG FIXES**
+ fixed race condition with duplicate file uploads

**FEATURES / ENHANCEMENTS**
+ added some basic documentation
+ refactored some parts

### v0.2.0
**BREAKING**
+ files are now served under `/d/<original_filename>` after a redirect from `/f/<short-id>`

**FEATURES / ENHANCEMENTS**
+ each command-line parameter now includes a short description
+ more elaborate information about the runtime configuration is now printed on startup
+ unnecessary sub-paths following a valid request path are no longer ignored
+ the new endpoint `/state/` returns a rendered list of all current uploads

### v0.1.0
MVP

## License

GNU Affero General Public License version 3, see [LICENSE](LICENSE)

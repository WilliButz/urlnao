Urlnao
======

Beware, project work in progress :)

Urlnao is an upload service for [file sharing with weechat-android](https://github.com/ubergeek42/weechat-android/wiki/File-sharing).

## Changelog

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

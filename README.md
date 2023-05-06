# Rust bookmark manager

## Build
Needs openssl: see https://docs.rs/openssl/latest/openssl/

## Running
### Development mode
In this mode, both the front-end and back-end changes are compiled whenever the sources files are saved. Execute
```sh
$ cargo install cargo-make
$ cargo make run-dev
```
The URL to use is http://localhost:8080.

### Production mode
In this mode, both the front-end and the back-end are compiled with `--release`.
```sh
$ cargo install cargo-make
$ cargo make run-prod
```
The URL to use is http://localhost:8000.

### Docker mode
In this mode, both the front-end and the back-end are compiled with `--release` and everything is packaged in a docker
image.
```sh
$ cargo install cargo-make
$ cargo make run-docker
```
The URL to use is http://localhost:8001.

## Configuration
### Environment variables
| name                | default                                   | description                                                  |
|---------------------|-------------------------------------------|--------------------------------------------------------------|
| `DATABASE_HOST`     | `localhost`                               | the database hostname                                        |
| `DATABASE_PORT`     | `5432`                                    | the database port                                            |
| `DATABASE_USERNAME` | `postgres`                                | the database username                                        |
| `DATABASE_PASSWORD` | `password`                                | the database password                                        |
| `DATABASE_NAME`     | `postgres`                                | the database name                                            |
| `HTTP_PORT`         | `3000`                                    | the HTTP port to listen to                                   |
| `HTTP_HOST`         | `0.0.0.0`                                 | the HTTP address to bind to                                  |
| `PUBLIC_URL`        | `http://$HTTP_HOST:$HTTP_PORT`            | public root url                                              |
| `ROOT_PATH`         | dev: `./webroot`; release: embedded files | the root folder of static files                              |
| `SMTP_HOST`         | `localhost`                               | the SMTP server hostname                                     |
| `SMTP_PORT`         | `25`                                      | the SMTP server port                                         |
| `SMTP_USERNAME`     | `username`                                | the SMTP username                                            |
| `SMTP_PASSWORD`     | `password`                                | the SMTP password                                            |
| `SMTP_FROM`         | `rbm@locahost`                            | the emails sender                                            |
| `REDIS_HOST`        | `localhost`                               | the redis hostname                                           |
| `REDIS_PORT`        | `6379`                                    | the redis port                                               |
| `REDIS_DB`          | `0`                                       | the redis database                                           |
| `COOKIE_SECRET`     | 64 random bytes                           | base64-encoded random bytes used to generate session cookies |
| `SESSION_TTL`       | `86400`                                   | session ttl, is seconds                                      |

## Planned features
* [x] CRUD functionality on bookmarks
* [x] Permalinks
* [x] Authentication
  * [ ] Invalidate all sessions after password change
  * [ ] Implement login throttling
  * [ ] Implement MFA
* [x] Link bookmarks to users
* [x] Tags on bookmarks
* [x] Private bookmarks
* [x] Pagination
* [x] Tag search
  * [ ] Find untagged bookmarks
* [x] Filtering by privacy
* [x] Full text search
* [x] Tag cloud
* [x] Sticky bookmarks
* [ ] Bookmarklet
* [ ] Export & Import bookmarks (incl. from Shaarli)
  * [x] Import from Shaarli's API
  * [ ] Import from Shaarli's JSON
  * [ ] Export to Shaarli's JSON
  * [ ] Export for backup
* [ ] Markdown support in bookmarks description
* [ ] Private bookmarks sharing
* [ ] Dead bookmarks detection & report
* [ ] Duplicate links report
  * [x] On add
  * [ ] Periodically
* [ ] Rename tag
* [ ] URL cleanup (utm_source=, fb=)
  * [x] Static params list
  * [ ] Dynamic params list (from database)
  * [ ] Advanced cleanup mode ([CleanLinks](https://github.com/Cimbali/CleanLinks/blob/master/addon/data/rules.json) and [link-cleaner](https://github.com/corbindavenport/link-cleaner/blob/main/js/shared.js))
* [ ] Shaarli compatible REST API: https://shaarli.readthedocs.io/en/master/REST-API/
* [ ] Notes
* [ ] Android app ([dimtion/Shaarlier](https://github.com/dimtion/Shaarlier))
* [ ] iOS app ([mro/ShaarliOS](https://github.com/mro/ShaarliOS))
* [ ] Read it later flags

## Resources
 * All the rust crates dependencies
 * [shaarli/Shaarli](https://github.com/shaarli/Shaarli) original idea, annoying query params
 * [mpchadwick/tracking-query-params-registry](https://github.com/mpchadwick/tracking-query-params-registry) crawled ignored get params
 * [Sh1d0w/clean-links](https://github.com/Sh1d0w/clean-links) more ignored get params

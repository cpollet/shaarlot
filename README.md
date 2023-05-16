# Shaarlot - a Shaarli-inspired bookmark manager written in rust

## Features
* Filtering by privacy, tag, words; pagination
* Sticky bookmarks
* Permalinks and QRCode
* Tag cloud
* Bookmarklet to add bookmarks easily
* Duplicate links detection when upon creation
* Removal of tracking URL query parameters (`utm_source=`, `fb=`, etc.)
* Import from Shaarli's API
* Demo mode with disabled account creation / update
* [More to come](https://github.com/cpollet/shaarlot/issues?q=is%3Aissue+is%3Aopen+label%3Aenhancement)

## Demo
A demo is available at https://demo.shaarlot.net.

## Bookmarklet
You can bookmark the current tab's URL with the following bookmarklet (just replace `HOSTNAME` with your actual hostname):
```javascript
javascript:(function(){var url=encodeURIComponent(window.location);window.open('https://HOSTNAME/bookmarks/~add?url='+url,'_blank','menubar=no,height=600,width=600,toolbar=no,scrollbars=yes,status=no,dialog=1');})();
```

## Build
Needs openssl: see https://docs.rs/openssl/latest/openssl/

## Configure
Shaarlot is configured via environment variables:

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
| `SMTP_FROM`         | `shaarlot@locahost`                       | the emails sender                                            |
| `REDIS_HOST`        | `localhost`                               | the redis hostname                                           |
| `REDIS_PORT`        | `6379`                                    | the redis port                                               |
| `REDIS_DB`          | `0`                                       | the redis database                                           |
| `COOKIE_SECRET`     | 64 random bytes                           | base64-encoded random bytes used to generate session cookies |
| `SESSION_TTL`       | `86400`                                   | session ttl, is seconds                                      |
| `DEMO`              | `false`                                   | demo mode if `true` (no account creation, no account update) |

## Run
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

## Resources
 * All the rust crates dependencies
 * [shaarli/Shaarli](https://github.com/shaarli/Shaarli) original idea, annoying query params
 * [mpchadwick/tracking-query-params-registry](https://github.com/mpchadwick/tracking-query-params-registry) crawled ignored get params
 * [Sh1d0w/clean-links](https://github.com/Sh1d0w/clean-links) more ignored get params

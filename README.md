# Rust bookmark manager

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
| name                | default                        | description                                    |
|---------------------|--------------------------------|------------------------------------------------|
| `DATABASE_HOST`     | `localhost`                    | the database hostname                          |
| `DATABASE_PORT`     | `5432`                         | the database port                              |
| `DATABASE_USERNAME` | `postgres`                     | the database username                          |
| `DATABASE_PASSWORD` | `password`                     | the database password                          |
| `DATABASE_NAME`     | `postgres`                     | the database name                              |
| `HTTP_PORT`         | `3000`                         | the HTTP port to listen to                     |
| `HTTP_HOST`         | `0.0.0.0`                      | the HTTP address to bind to                    |
| `PUBLIC_URL`        | `http://$HTTP_HOST:$HTTP_PORT` | public root url                                |
| `ROOT_PATH`         | `./webroot`                    | the root folder of static files                |
| `ASSETS_URL`        | `/assets`                      | the root url under which `ROOT_PATH` is served |
| `SMTP_HOST`         | `localhost`                    | the SMTP server hostname                       |
| `SMTP_PORT`         | `25`                           | the SMTP server port                           |
| `SMTP_USERNAME`     | `username`                     | the SMTP username                              |
| `SMTP_PASSWORD`     | `password`                     | the SMTP password                              |
| `SMTP_FROM`         | `rbm@locahost`                 | the emails sender                              |
| `REDIS_HOST`        | `localhost`                    | the redis hostname                             |
| `REDIS_PORT`        | `6379`                         | the redis port                                 |
| `REDIS_DB`          | `0`                            | the redis database                             |
| `COOKIE_SECRET`     | 64 random bytes                | random bytes used to generate session cookies  |
| `SESSION_TTL`       | `86400`                        | session ttl, is seconds                        |

## Planned features
* [x] CRUD functionality on links
* [x] Permalinks
* [ ] Authentication
* [ ] Private links
* [ ] Read it later flags
* [ ] Tags on links
* [ ] Pagination
* [ ] Search & filtering
* [ ] Tag cloud
* [ ] Private links sharing
* [ ] Markdown support in links description
* [ ] Export & Import links (incl. from Shaarli)
* [ ] Dead links detection & report
* [ ] URL cleanup (utm_source=, fb=)
* [ ] Shaarli compatible REST API: https://shaarli.readthedocs.io/en/master/REST-API/

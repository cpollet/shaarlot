# Rust bookmark manager

## Running
### Development mode
In this mode, both the front-end and back-end changes are compiled whenever the sources files are saved.
It is with `./dev.sh`. The URL to use is http://localhost:8080.

### Production mode
In this mode, both the front-end and the back-end are compiled with `--release`.
It is started with `./prod.sh` The URL to use is http://localhost:8000.

### Docker mode
In this mode, both the front-end and the back-end are compiled with `--release` and everything is packaged in a docker
image.
It is started with `./docker.sh` The URL to use is http://localhost:8001.

## Configuration
### Environment variables
| name                | default     | description                                     |
|---------------------|-------------|-------------------------------------------------|
| `DATABASE_HOST`     | `localhost` | the database hostname                           |
| `DATABASE_PORT`     | `5432`      | the database port                               |
| `DATABASE_USERNAME` | `postgres`  | the database username                           |
| `DATABASE_PASSWORD` | `password`  | the database password                           |
| `DATABASE_NAME`     | `postgres`  | the database name                               |
| `HTTP_PORT`         | `3000`      | the HTTP port to listen to                      |
| `HTTP_HOST`         | `0.0.0.0`   | the HTTP address to bind to                     |
| `ROOT_PATH`         | `./webroot` | the root folder of static files                 |
| `ASSETS_URL`        | `/assets`   | the root url under which `ROOT_PATH` is served | 

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

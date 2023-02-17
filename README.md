# Rust bookmark manager

## Running
### Development mode
In this mode, both the front-end and back-end changes are compiled whenever the sources files are saved.
It is with `./dev.sh`. The URL to use is http://localhost:3000.

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
| `ASSETS_URL`        | `/assets`   | the root url under wihich `ROOT_PATH` is served | 

## Planned features
* [ ] CRUD functionnality on links
* [ ] Tags on links
* [ ] Tag cloud
* [ ] Search
* [ ] Markdown support in links description
* [ ] Export & Import links (inlc. from shaarli)
* [ ] Dead links detection & report
* [ ] Authentication
* [ ] Permalinks
* [ ] Pagination
* [ ] URL cleanup (utm_source=, fb=)
* [ ] Shaarli compatible REST API: https://shaarli.readthedocs.io/en/master/REST-API/
* [ ] Read it later flags

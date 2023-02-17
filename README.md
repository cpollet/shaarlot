# Rust bookmark manager


## Configuration
### Environment variables
| name                | default     | description                       |
|---------------------|-------------|-----------------------------------|
| `DATABASE_HOST`     | `localhost` | the database hostname             |
| `DATABASE_PORT`     | `5432`      | the database port                 |
| `DATABASE_USERNAME` | `postgres`  | the database username             |
| `DATABASE_PASSWORD` | `password`  | the database password             |
| `DATABASE_NAME`     | `postgres`  | the database name                 |
| `HTTP_PORT`         | `3000`      | the HTTP port to listen to        |
| `HTTP_HOST`         | `0.0.0.0`   | the HTTP address to bind to       |
| `ROOT_PATH`         | `./dist`    | the root folder of static files   |

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

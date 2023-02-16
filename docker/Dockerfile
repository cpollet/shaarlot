FROM debian:bullseye-slim

COPY rbm /usr/local/bin/
COPY dist /usr/local/bin/dist/

EXPOSE 3000

WORKDIR /usr/local/bin
CMD [ "rbm" ]
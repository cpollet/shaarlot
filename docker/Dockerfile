FROM debian:bullseye-slim

COPY target/rbm /usr/local/bin/
COPY target/webroot /usr/local/bin/webroot

EXPOSE 3000

WORKDIR /usr/local/bin
CMD [ "rbm" ]
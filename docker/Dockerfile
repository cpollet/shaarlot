FROM debian:bullseye-slim

RUN apt-get update ; \
    apt-get install -y tini ; \
    apt-get clean ; \
    rm -rf /var/lib/apt/lists/*

COPY target /usr/local/bin/

EXPOSE 3000

WORKDIR /usr/local/bin
CMD [ "/usr/bin/tini", "rbm" ]
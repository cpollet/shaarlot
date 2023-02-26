#!/usr/bin/env bash

source .env

RBM_BUILD_ONLY=1 ./docker.sh

docker push cpollet/rbm
curl "$WEBHOOK_URL"
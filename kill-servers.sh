#!/bin/bash

lsof -i tcp:3000 | grep LISTEN | awk '{print $2}' | xargs kill
lsof -i tcp:8080 | grep LISTEN | awk '{print $2}' | xargs kill

pgrep cargo-make | xargs kill 2>/dev/null
pgrep cargo-watch | xargs kill 2>/dev/null

lsof -i tcp:3000 | grep LISTEN
lsof -i tcp:8080 | grep LISTEN
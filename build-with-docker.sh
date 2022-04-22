#!/usr/bin/env bash

# Create docker image
docker build -t diadro -f Dockerfile.finish .

# Start image
docker run --name "diadro" diadro
# Copy file from container to folder
docker cp "$(docker ps -a -q -f "name=diadro")":/diadro release/diadro
# Stop container
docker stop "$(docker ps -a -q -f "name=diadro")"
# Drop container
docker rm "$(docker ps -a -q -f "name=diadro")"

# Save docker image to file with compression by gzip
# Uncomment if you want to save base image
#mkdir release/docker-image
#docker save diadro:latest | gzip > release/docker-image/diadro.tar.gz

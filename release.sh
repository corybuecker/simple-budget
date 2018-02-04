#!/usr/bin/env bash

set -ex

IMAGE=gcr.io/cluster-1249/simple-budget

git pull

version=`date +%s`
echo "version: $version"

docker build -t $IMAGE:$version .
docker tag $IMAGE:$version $IMAGE:latest

gcloud docker -- push $IMAGE:latest
gcloud docker -- push $IMAGE:$version

kubectl set image deployments/simple-budget-simple-budget simple-budget=$IMAGE:$version

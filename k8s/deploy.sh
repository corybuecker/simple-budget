#!/bin/bash

set -e

if [ ! -f deployment.yaml ]; then
  echo "can only run inside the K8s directory"
  exit 1
fi

read -p "Domain: " domain
read -p "Image URL: " image

echo $domain
echo $image

rm -rf ./output
mkdir -p ./output/container_cast
cp -R *.yaml ./output/
cp -R container_cast/*.yaml ./output/container_cast/

echo "s/\${IMAGE}/$image/g"

sed -i '' "s/\${HOST}/$domain/g" output/ingress.yaml
sed -i '' "s/\${IMAGE}/$image/g" output/deployment.yaml
sed -i '' "s/\${IMAGE}/$image/g" output/container_cast/deployment.yaml

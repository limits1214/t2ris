#!/bin/bash
set -e

docker build \
  -t ghcr.io/limits1214/t3tris-front:latest \
  --push \
  .

# docker buildx build \
#   --platform linux/arm64 \
#   -t ghcr.io/limits1214/t3tris-front:latest \
#   --push \
#   .

# docker build . -t ghcr.io/limits1214/t3tris-front:latest --platform linux/arm64
# docker push ghcr.io/limits1214/t3tris-front:latest
#!/bin/sh

docker buildx build -t vadorovsky/anchor-heap-experiments .
docker run --rm -it \
  -v solana-config:/home/node/.config/solana \
  -v .:/home/node/anchor-heap-experiments \
  vadorovsky/anchor-heap-experiments \
  bash

default:
    just --list

types:
    cd server && tsync -i src/message.rs -o ../web/src/message.ts
    cd web && yarn generate-zod

build:
    nix build && ./result | podman load

deploy: build
    podman push localhost/werwolf docker://registry.fly.io/werwolf:latest
    flyctl deploy -i registry.fly.io/werwolf:latest

dev-server:
    cd server && cargo run

dev-web:
    cd web && yarn dev
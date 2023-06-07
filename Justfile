default:
    just --list

types:
    cd server && tsync -i src/message.rs -o ../web/src/message.ts
    cd web && yarn generate-zod


dev-server:
    cd server && cargo shuttle run

dev-web:
    cd web && yarn dev

#deploy: types
#    cd web && yarn build
#    cd server && cargo shuttle deploy
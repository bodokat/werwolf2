types:
    typical generate types.t --rust server/src/message.rs --typescript web/src/message.ts



dev-server:
    cd server && cargo shuttle run

dev-web:
    cd web && yarn dev

deploy: types
    cd web && yarn build
    cd server && cargo shuttle deploy
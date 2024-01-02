# Wallet Server

## Migrate database
```
DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/wallet_server" cargo run --bin wallet_server_migration -- fresh
```

## Generate entities
```
sea-orm-cli generate entity -o wallet_server/src/entity --database-url "postgres://postgres:postgres@127.0.0.1:5432/wallet_server"
```

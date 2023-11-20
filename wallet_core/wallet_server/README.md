# Wallet Server

## Migrate database
```
sea-orm-cli migrate --database-url "postgres://postgres:postgres@127.0.0.1:5432/wallet_server"
```

## Generate entities
```
sea-orm-cli generate entity -o wallet_server/src/entity --database-url "postgres://postgres:postgres@127.0.0.1:5432/wallet_server"
```

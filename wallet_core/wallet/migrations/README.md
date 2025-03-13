# Wallet Sqlite

This is the `sea-orm` migrations crate for the wallet, which is responsible for
creating the `sqlite` database "on the mobile device", i.e., it exists as part
of `libwallet.so` which exists as part of `wallet_app`. For debugging purposes
you can create the `wallet.sqlite` database locally. This is convenient when
you want to inspect what the database looks like.

## Create the database locally

To create the database locally in the current `migrations` folder, you can do
the following:

```shell
touch wallet.sqlite
cargo run -- fresh -u sqlite://wallet.sqlite
cargo run -- status -u sqlite://wallet.sqlite
```

After doing the above, you'll see a `wallet.sqlite` file which you can open
with sqlite3 command-line or with something like JetBrains DataGrip.

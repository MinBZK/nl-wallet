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


## A note about UUIDv7 primary keys and order

We use `uuid-rs` with the `v7` feature to have order-preserving UUIDs as primary
keys. Specifically, we use the `now_v7()` function to generate a UUID primary
key/ This is an alternative to [`Uuid::new_v7`] that uses the current system
time as a source timestamp. All UUIDs generated through this method by the same
process are guaranteed to be ordered by their creation.

From the documentation on the `ContextV7` struct:

(ContextV7 is) an unsynchronized, reseeding counter that produces 42-bit values.

This type works by:

1. Reseeding the counter each millisecond with a random 41-bit value. The 42nd
   bit is left unset so the counter can safely increment over the millisecond.

2. Wrapping the counter back to zero if it overflows its 42-bit storage and
   adding a millisecond to the timestamp.

The counter can use additional sub-millisecond precision from the timestamp to
better synchronize UUID sorting in distributed systems. In these cases, the
additional precision is masked into the left-most 12 bits of the counter. The
counter is still reseeded on each new millisecond, and incremented within the
millisecond. This behavior may change in the future. The only guarantee is
monotonicity.

This type can be used when constructing version 7 UUIDs. When used to construct
a version 7 UUID, the 42-bit counter will be padded with random data. This type
can be used to maintain ordering of UUIDs *within* the same millisecond.


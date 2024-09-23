# GBA-V Haalcentraal converter

If the gba-hc-converter is to be used to serve preloaded gba-v-responses, the `preloaded.xml_path` setting needs to
point to a path containing encrypted gba-v xml responses. See the settings below for more details.

## Settings

### Generate random symmetric key

The gba-hc-converter needs a `preloaded.symmetric_key` configured for decrypting the gba-v responses. This can be
manually
generated using:

    openssl rand -base64 32 > symmetric.key

### Encrypting the GBA-V responses

If a gba-v response needs to be manually encrypted, the following command can be used:

    mkdir resources/encrypted-gba-v-responses
    cargo run --bin gba_encrypt -- \
                --basename "999991772" \
                --output "resources/encrypted-gba-v-responses" \
                "gba-v-responses/999991772.xml"

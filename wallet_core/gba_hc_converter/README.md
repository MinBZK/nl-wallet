# GBA-V Haalcentraal converter

If the gba-hc-converter is to be used to serve preloaded gba-v-responses, the `preloaded.xml_path` setting needs to
point to a path containing encrypted gba-v xml responses. See the settings below for more details.

## Settings

### Generate random keys

The gba-hc-converter needs a `preloaded.encryption_key` configured for decrypting the gba-v responses. This can be
manually generated using:

    openssl rand -hex 32 > encryption.key

In addition, a `preloaded.hmac_key` is necessary for hashing the BSN so it can be used as filename:

    openssl rand -hex 64 > hmac.key

### Prefetching GBA-V responses

For manually prefetching a gba-v response, the following binary can be used:

    mkdir output_dir
    cargo run --bin gba_fetch -- --output output_dir

### Encrypting the GBA-V responses

If a gba-v response needs to be manually encrypted, the following command can be used:

    mkdir resources/encrypted-gba-v-responses
    cargo run --bin gba_encrypt -- \
                --basename "999991772" \
                --output "resources/encrypted-gba-v-responses" \
                "gba-v-responses/999991772.xml"

### Calculating HMAC manually

The filename of the encrypted gba-v can be calculated manually as follows:

    echo -n "999991772" | openssl dgst -hmac "<hmac_key>" -sha256

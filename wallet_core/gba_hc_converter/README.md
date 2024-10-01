# GBA-V Haalcentraal converter

The gba-hc-converter is a web server that converts GBA-V XML responses to a
JSON format that the HaalCentraal service can use.

If the gba-hc-converter is to be used to serve preloaded GBA-V-responses, the
`preloaded.xml_path` setting needs to point to a path containing encrypted GBA-V
XML responses. See the settings below for more details.

## Settings

### Generate random keys

The gba-hc-converter needs a `preloaded.encryption_key` configured for
decrypting the GBA-V responses. This can be
manually generated using:

    openssl rand -hex 32 > encryption.key

In addition, a `preloaded.hmac_key` is necessary for hashing the BSN so it can
be used as filename:

    openssl rand -hex 64 > hmac.key

## Prefetching GBA-V responses

There is a specific requirement that prevents using a "live" GBA-V connection in
the gba-hc-converter in certain environments. Therefore, a binary is offered
that allows prefetching GBA-V responses separately. The preloaded GBA-V data is
stored in a location that the gba-hc-converter can access. The gba-hc-converter
can be configured in such a way that it cannot connect to the GBA-V, but only
serves preloaded data.

The preloaded data is encrypted at rest using the AES-GCM Authenticated
Encryption with Associated Data (AEAD) cipher. Encrypting the data at rest
protects from unwanted data leakage, for instance accidental (off-site) backups.
Both preload binary and gba-hc-converter are in posession of the encryption key.
The preload binary writes that encrypted data to a shared storage location, from
which the gba-hc-converter is only allowed to read. The encrypted data is stored
having a specific filename. The filename is constructed as the signature from
the BSN and a secret key using HMAC-SHA256. This way, when the gba-hc-converter
receives a request to fetch data for a specific BSN, it calculates the signature
of the BSN (using the method described above) and checks if there is a file
having that signature as filename on the filesystem. If so, the preloaded data
is decrypted and served. If not, it can either return a not found response or
fetch the data from GBA-V, depending on how it is configured.

For manually prefetching a GBA-V response, the following binary can be used:

    mkdir output_dir
    cargo run --bin gba_fetch -- --output output_dir

### Encrypting the GBA-V responses

If a GBA-V response needs to be manually encrypted, the following command can be
used:

    mkdir resources/encrypted-gba-v-responses
    cargo run --bin gba_encrypt -- \
                --basename "999991772" \
                --output "resources/encrypted-gba-v-responses" \
                "gba-v-responses/999991772.xml"

### Calculating HMAC manually

The filename of the encrypted GBA-V can be calculated manually as follows:

    echo -n "999991772" | openssl mac -digest sha256 -macopt hexkey:<hmac_key> HMAC

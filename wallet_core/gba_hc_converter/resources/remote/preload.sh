#!/bin/sh
set -euo

cleanup() {
    trap - INT HUP TERM
    rm -f /tmp/client_cert_key.der /tmp/bsn_zoeken.xml
    exit
}

trap cleanup INT HUP TERM

input="$1"
iter=

# Curl doesn't support the DER format for the cacert option, so we construct a PEM file manually.
printf "%s\n" "-----BEGIN CERTIFICATE-----" "$(echo "$GBAV_TRUST_ANCHOR" | fold -w64)" "-----END CERTIFICATE-----"  > /tmp/trust_anchor.pem
echo "$GBAV_CLIENT_CERT" | base64 -d > /tmp/client_cert.der
echo "$GBAV_CLIENT_CERT_KEY" | base64 -d > /tmp/client_cert_key.der

while [ "$input" != "$iter" ] ;do
    # Extract the substring from start of string up to delimiter.
    iter="${input%%,*}"
    # Delete this first "element" AND its separator from $input.
    input="${input#"$iter",}"

    # Fill in the bsn in the template that will be sent to the GBA-V.
    sed s/\{\{bsn\}\}/"$iter"/g /tmp/bsn_zoeken_template.xml > /tmp/bsn_zoeken.xml

    curl -v --tls-max 1.2 --user "${GBAV_USERNAME}:${GBAV_PASSWORD}" \
      --cacert /tmp/trust_anchor.pem \
      --cert /tmp/client_cert.der --cert-type DER \
      --key /tmp/client_cert_key.der --key-type DER \
      --header "Accept-Charset: UTF-8" --header "Content-Type: application/xml;charset=UTF-8" \
      --data-binary @/tmp/bsn_zoeken.xml \
      --output /data/"$iter".xml \
      "$GBAV_ADHOC_URL"
done

cleanup

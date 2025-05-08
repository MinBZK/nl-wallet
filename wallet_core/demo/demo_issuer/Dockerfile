FROM scratch

ARG CSP_SCRIPT_DEMO_ISSUER
ARG CSP_STYLE_DEMO_ISSUER
ARG WALLET_WEB_FILENAME
ARG WALLET_WEB_SHA256
LABEL csp_script_demo_issuer="$CSP_SCRIPT_DEMO_ISSUER"
LABEL csp_style_demo_issuer="$CSP_STYLE_DEMO_ISSUER"
LABEL wallet_web_filename="$WALLET_WEB_FILENAME"
LABEL wallet_web_sha256="$WALLET_WEB_SHA256"

COPY ./target/x86_64-unknown-linux-musl/release/demo_issuer .
COPY demo/demo_issuer/assets/ ./assets/
COPY demo/demo_utils/assets/ ./assets/

USER 1000
ENTRYPOINT ["./demo_issuer"]

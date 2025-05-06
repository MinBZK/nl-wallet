FROM scratch

ARG CSP_SCRIPT_DEMO_RP_BASE64
ARG CSP_STYLE_DEMO_RP_BASE64
ARG WALLET_WEB_FILENAME
ARG WALLET_WEB_SHA256
LABEL csp_script_demo_rp_base64="$CSP_SCRIPT_DEMO_RP_BASE64"
LABEL csp_style_demo_rp_base64="$CSP_STYLE_DEMO_RP_BASE64"
LABEL wallet_web_filename="$WALLET_WEB_FILENAME"
LABEL wallet_web_sha256="$WALLET_WEB_SHA256"

COPY ./target/x86_64-unknown-linux-musl/release/demo_relying_party .
COPY demo/demo_relying_party/assets/ ./assets/

USER 1000
ENTRYPOINT ["./demo_relying_party"]

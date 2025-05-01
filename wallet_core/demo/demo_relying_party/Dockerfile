FROM scratch

ARG CSP_SCRIPT_DEMO_RP
ARG CSP_STYLE_DEMO_RP
ARG WALLET_WEB_FILENAME
ARG WALLET_WEB_SHA256
LABEL csp_script_demo_rp="$CSP_SCRIPT_DEMO_RP"
LABEL csp_style_demo_rp="$CSP_STYLE_DEMO_RP"
LABEL wallet_web_filename="$WALLET_WEB_FILENAME"
LABEL wallet_web_sha256="$WALLET_WEB_SHA256"

COPY ./target/x86_64-unknown-linux-musl/release/demo_relying_party .
COPY demo/demo_relying_party/assets/ ./assets/
COPY demo/demo_utils/assets/ ./assets/

USER 1000
ENTRYPOINT ["./demo_relying_party"]

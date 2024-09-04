FROM scratch

ARG CSP_SCRIPT_MRP_BASE64
ARG CSP_STYLE_MRP_BASE64
ARG WALLET_WEB_FILENAME
ARG WALLET_WEB_SHA256
LABEL csp_script_mrp_base64="$CSP_SCRIPT_MRP_BASE64"
LABEL csp_style_mrp_base64="$CSP_STYLE_MRP_BASE64"
LABEL wallet_web_filename="$WALLET_WEB_FILENAME"
LABEL wallet_web_sha256="$WALLET_WEB_SHA256"

COPY ./target/x86_64-unknown-linux-musl/release/mock_relying_party .
COPY mock_relying_party/assets/ ./assets/

USER 1000
ENTRYPOINT ["./mock_relying_party"]

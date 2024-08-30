FROM scratch
COPY ./target/x86_64-unknown-linux-musl/release/gba_hc_converter .
COPY ./gba_hc_converter/resources/remote/bsn_zoeken_template.xml /resources/remote/bsn_zoeken_template.xml
COPY ./gba_hc_converter/resources/stamdata /resources/stamdata
USER 1000
CMD ["./gba_hc_converter"]

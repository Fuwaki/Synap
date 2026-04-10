FROM rustlang/rust:nightly-alpine AS builder

RUN apk add --no-cache bash curl npm libc-dev binaryen
RUN npm install -g sass
RUN curl --proto '=https' --tlsv1.3 -LsSf \
    https://github.com/leptos-rs/cargo-leptos/releases/latest/download/cargo-leptos-installer.sh \
    | sh
RUN rustup target add wasm32-unknown-unknown

WORKDIR /work
COPY Cargo.toml Cargo.lock ./
COPY cli ./cli
COPY core ./core
COPY coreffi ./coreffi
COPY desktop ./desktop
COPY web ./web
COPY xtask ./xtask

WORKDIR /work/web
ENV CARGO_TERM_COLOR=always
ENV RUSTFLAGS="-C link-arg=-s"
RUN cargo leptos build --release -vv

RUN mkdir -p /out/app /out/data \
    && cp /work/target/release/web /out/app/web 2>/dev/null \
        || cp /work/web/target/release/web /out/app/web \
    && cp -r /work/target/site /out/app/site 2>/dev/null \
        || cp -r /work/web/target/site /out/app/site

FROM scratch AS runner

WORKDIR /app
COPY --from=builder /out/ /

ENV RUST_LOG=info
ENV LEPTOS_SITE_ADDR=0.0.0.0:8080
ENV LEPTOS_SITE_ROOT=/app/site
ENV SYNAP_WEB_DB=/data/synap-web.redb

EXPOSE 8080

CMD ["/app/web"]

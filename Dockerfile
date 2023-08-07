FROM rust:alpine as build-backend
WORKDIR /build

RUN apk add musl-dev

COPY ./Cargo.lock ./Cargo.toml ./
COPY ./src ./src

RUN cargo build --release

FROM scratch
WORKDIR /app

ENV PATH="$PATH:/app/bin"

COPY --from=build-backend /build/target/release/apt-cache /app/bin/apt-cache

ENV LISTEN_ADDRESS=0.0.0.0:80
ENV CACHE_PATH=/cache
ENV CONFIG_PATH=/config.toml
EXPOSE 80

VOLUME [ "/cache" ]
VOLUME [ "/config.toml" ]
CMD [ "/app/bin/apt-cache" ]
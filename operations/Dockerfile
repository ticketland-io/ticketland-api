FROM rust:1.65 as builder

ARG GITHUB_TOKEN
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

RUN git config --global url."https://${GITHUB_TOKEN}@github.com/".insteadOf "https://github.com/"

RUN USER=root cargo new --bin ticketland-api-dist
COPY ./src/api ./ticketland-api-dist
COPY ./Cargo.lock ./ticketland-api-dist
WORKDIR ./ticketland-api-dist
RUN cargo build --release

RUN git config --global --unset url."https://${GITHUB_TOKEN}@github.com/".insteadOf

FROM debian:bullseye-slim
ARG APP=/usr/src/app

EXPOSE 8000

ENV TZ=Etc/UTC

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /ticketland-api-dist/target/release/ticketland-api ${APP}/ticketland-api

USER $APP_USER
WORKDIR ${APP}

ENV ENV=production
CMD ["./ticketland-api"]

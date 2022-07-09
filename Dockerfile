FROM rust:latest as backend

RUN update-ca-certificates

ENV USER=baryonicbackend
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /baryonic

COPY ./ .

RUN cargo build --release

FROM ubuntu:latest

COPY --from=backend /etc/passwd /etc/passwd
COPY --from=backend /etc/group /etc/group

WORKDIR /baryonic

COPY --from=backend /baryonic/target/release/baryonic-game-backend ./

USER baryonicbackend:baryonicbackend

CMD ["/baryonic/baryonic-game-backend"]
EXPOSE 8080/tcp

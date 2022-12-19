FROM rust:1.65 as builder

WORKDIR /centinote
COPY . .
RUN cargo install --path .

FROM rockylinux:9-minimal
COPY --from=builder /usr/local/cargo/bin/centinote /usr/local/bin/centinote
COPY sql /usr/local/share/centinote/sql
COPY html /usr/local/share/centinote/html
CMD ["centinote"]

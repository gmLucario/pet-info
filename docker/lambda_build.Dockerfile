FROM ghcr.io/cargo-lambda/cargo-lambda:latest as builder

WORKDIR /build

COPY . .

RUN cargo lambda build --release --arm64 --output-format zip

FROM scratch

COPY --from=builder /build/target/lambda/send-reminders/bootstrap.zip /

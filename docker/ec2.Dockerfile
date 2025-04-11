FROM public.ecr.aws/amazonlinux/amazonlinux:2023-minimal AS builder

RUN dnf update -y
RUN dnf install -y rust cargo make automake gcc gcc-c++ kernel-devel git openssl openssl-devel

WORKDIR /build

COPY . .

RUN cargo build -r

FROM scratch
ARG SOURCE_NAME

COPY --from=builder /build/target/release/${SOURCE_NAME} /

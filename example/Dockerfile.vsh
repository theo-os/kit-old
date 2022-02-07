FROM docker.io/clux/muslrust:latest as build

RUN apt update -y && \
    apt install -y \
    clang-3.9 \
    libclang-3.9-dev \
    llvm-3.9-dev \
    patch perl

WORKDIR /usr/src/theos/vsh

RUN git clone --depth 1 https://github.com/Vaimer9/vsh .

RUN cargo build --target=x86_64-unknown-linux-musl --release

FROM scratch

COPY --from=build /usr/src/theos/vsh/target/x86_64-unknown-linux-musl/release/vsh /bin/

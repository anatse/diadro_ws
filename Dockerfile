# ------------------------------------------------------------------------------
# Создаем образ для сборки приложения
# дополнительная информация https://shaneutt.com/blog/rust-fast-small-docker-image-builds/
# ------------------------------------------------------------------------------
FROM rust:latest

RUN apt-get update && \
  apt-get upgrade -y && \
  apt-get install -y \
  ca-certificates \
  musl-dev \
  musl-tools \
  file \
  nano \
  git \
  zlib1g-dev \
  cmake \
  make \
  clang \
  curl \
  pkgconf \
  linux-headers-amd64 \
  xutils-dev \
  libpq-dev \
  libssl-dev \
  libclang-dev libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev \
  jq \
  binaryen

RUN ln -s /usr/include/x86_64-linux-gnu/asm /usr/include/x86_64-linux-musl/asm && \
    ln -s /usr/include/asm-generic /usr/include/x86_64-linux-musl/asm-generic && \
    ln -s /usr/include/linux /usr/include/x86_64-linux-musl/linux

WORKDIR /
RUN mkdir /musl
RUN wget https://github.com/openssl/openssl/archive/OpenSSL_1_1_1n.tar.gz
RUN tar zxvf OpenSSL_1_1_1n.tar.gz
WORKDIR openssl-OpenSSL_1_1_1n/
RUN CC="musl-gcc -fPIE -pie" ./Configure no-shared no-async --prefix=/musl --openssldir=/musl/ssl linux-x86_64
RUN make depend
RUN make -j$(nproc)
RUN make install

ENV PKG_CONFIG_ALLOW_CROSS=1
ENV OPENSSL_STATIC=true
ENV OPENSSL_DIR=/musl

WORKDIR /usr/src/diadro

COPY Cargo.toml .
COPY Cargo.lock .

COPY . /usr/src/diadro

RUN rustup target add x86_64-unknown-linux-musl

# RUN cargo update
RUN cargo build --target x86_64-unknown-linux-musl --release --package dserver
WORKDIR /usr/src/diadro/diadro

RUN cargo install -f wasm-bindgen-cli
RUN rustup target add wasm32-unknown-unknown
RUN ./build_web.sh

CMD ["sh"]

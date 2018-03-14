#FROM japaric/x86_64-unknown-linux-gnu:v0.1.4
FROM ubuntu:14.04

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    software-properties-common python-software-properties && \
    add-apt-repository ppa:segf4ult/zeromq -y && \
    apt-get update && \
    apt-get install -y --no-install-recommends  \
    ca-certificates \
    gcc \
    libc6-dev \
    make \
    pkg-config \
    libavahi-compat-libdnssd-dev:armhf \
    libsodium-dev:armhf \
    libtag1-dev:armhf libtagc0-dev:armhf

RUN curl -fsSL https://raw.githubusercontent.com/japaric/cross/master/docker/xargo.sh > xargo.sh && \
    bash /xargo.sh

RUN curl -fsSL https://raw.githubusercontent.com/japaric/cross/master/docker/cmake.sh > cmake.sh && \
    apt-get purge --auto-remove -y cmake && \
    bash /cmake.sh 2.8.11

RUN curl -fsSL https://raw.githubusercontent.com/japaric/cross/master/docker/openssl.sh > openssl.sh && \
    curl -fsSL https://raw.githubusercontent.com/japaric/cross/master/docker/qemu.sh > qemu.sh && \
    apt-get install -y --no-install-recommends \
    g++-arm-linux-gnueabihf \
    libc6-dev-armhf-cross && \
    bash /openssl.sh linux-armv4 arm-linux-gnueabihf- && \
    bash /qemu.sh arm

ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc \
    CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUNNER=qemu-arm \
    CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc \
    CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++ \
    OPENSSL_DIR=/openssl \
    OPENSSL_INCLUDE_DIR=/openssl/include \
    OPENSSL_LIB_DIR=/openssl/lib \
    QEMU_LD_PREFIX=/usr/arm-linux-gnueabihf \
    RUST_TEST_THREADS=1

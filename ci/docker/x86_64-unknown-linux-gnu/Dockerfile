#FROM japaric/x86_64-unknown-linux-gnu:v0.1.4
FROM ubuntu:14.04

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    apt-get install software-properties-common python-software-properties -y && \
    apt-get install libavahi-compat-libdnssd-dev:amd64 -y && \
    apt-get install libsodium-dev:amd64 -y && \
    apt-get install libtag1-dev:amd64 libtagc0-dev:amd64 -y
    ca-certificates \
    cmake \
    gcc \
    libc6-dev \
    make \
    pkg-config

RUN curl -fsSL https://raw.githubusercontent.com/japaric/cross/master/docker/xargo.sh && \
    bash /xargo.sh

RUN curl -fsSL https://raw.githubusercontent.com/japaric/cross/master/docker/cmake.sh && \
    apt-get purge --auto-remove -y cmake && \
    bash /cmake.sh 2.8.11

RUN curl -fsSL https://raw.githubusercontent.com/japaric/cross/master/docker/openssl.sh && \
RUN apt-get install -y --no-install-recommends \
    g++ \
    zlib1g-dev && \
    bash /openssl.sh linux-x86_64

ENV OPENSSL_DIR=/openssl \
    OPENSSL_INCLUDE_DIR=/openssl/include \
    OPENSSL_LIB_DIR=/openssl/lib

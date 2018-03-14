#FROM japaric/x86_64-unknown-linux-gnu:v0.1.4
FROM ubuntu:14.04

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    software-properties-common python-software-properties && \
    add-apt-repository ppa:james-page/0mq -y && \
    apt-get update && \
    apt-get install -y --no-install-recommends  \
    ca-certificates \
    cmake \
    gcc \
    libc6-dev \
    make \
    pkg-config \
    libavahi-compat-libdnssd-dev:amd64 \
    libsodium-dev:amd64 \
    libtag1-dev:amd64 libtagc0-dev:amd64 \
    curl

RUN curl -fsSL https://raw.githubusercontent.com/japaric/cross/master/docker/xargo.sh > xargo.sh && \
    bash /xargo.sh

RUN curl -fsSL https://raw.githubusercontent.com/japaric/cross/master/docker/cmake.sh > cmake.sh && \
    apt-get purge --auto-remove -y cmake && \
    bash /cmake.sh 2.8.11

RUN curl -fsSL https://raw.githubusercontent.com/japaric/cross/master/docker/openssl.sh > openssl.sh && \
    apt-get install -y --no-install-recommends \
    g++ \
    zlib1g-dev && \
    bash /openssl.sh linux-x86_64

ENV OPENSSL_DIR=/openssl \
    OPENSSL_INCLUDE_DIR=/openssl/include \
    OPENSSL_LIB_DIR=/openssl/lib

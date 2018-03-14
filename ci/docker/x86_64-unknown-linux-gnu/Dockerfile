FROM ubuntu:14.04

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    cmake \
    gcc \
    libc6-dev \
    make \
    pkg-config

COPY xargo.sh /
RUN bash /xargo.sh

COPY cmake.sh /
RUN apt-get purge --auto-remove -y cmake && \
    bash /cmake.sh 2.8.11

COPY openssl.sh /
RUN apt-get install -y --no-install-recommends \
    g++ \
    zlib1g-dev && \
    bash /openssl.sh linux-x86_64

ENV OPENSSL_DIR=/openssl \
    OPENSSL_INCLUDE_DIR=/openssl/include \
    OPENSSL_LIB_DIR=/openssl/lib

#FROM japaric/x86_64-unknown-linux-gnu:v0.1.4
RUN apt-get install software-properties-common python-software-properties -y && \
    apt-get install libavahi-compat-libdnssd-dev:amd64 -y && \
    apt-get install libsodium-dev:amd64 -y && \
    apt-get install libtag1-dev:amd64 libtagc0-dev:amd64 -y

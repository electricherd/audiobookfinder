FROM multiarch/debian-debootstrap:armel-buster-slim

RUN apt-get update && apt-get install -y \
    apt-utils \
    build-essential \
    ca-certificates \
    clang \
    cmake 

RUN apt-get install -y apt-transport-https ca-certificates curl software-properties-common
ARG RUST_VER=1.32.0

# Make sure rustup and cargo are in PATH
ENV PATH "~/.cargo/bin:$PATH"
#RUN export SSL_CERT_FILE=/etc/ssl/ca-bundle.pem
# Install rustup, skip latest toolchain and get a specific version
RUN curl --insecure --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > ./rustup.sh
RUN chmod a+x ./rustup.sh
RUN RUSTUP_USE_REQWEST=1 SSL_CERT_DIR=/usr/local/etc/ssl/certs ./rustup.sh -s -y --default-toolchain none && \
       ~/.cargo/bin/rustup default $RUST_VER
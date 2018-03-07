FROM japaric/x86_64-unknown-linux-gnu:v0.1.4

RUN \
 # dpkg --add-architecture x64_64 && \
    apt-get install -qq libavahi-compat-libdnssd-dev -y && \
    add-apt-repository ppa:segf4ult/zeromq -y && \
    apt-get update -qq &&\
    apt-get install libsodium-dev

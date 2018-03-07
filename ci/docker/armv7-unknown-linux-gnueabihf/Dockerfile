FROM japaric/armv7-unknown-linux-gnueabihf:v0.1.4

RUN apt-get install software-properties-common && \
    add-apt-repository ppa:segf4ult/zeromq && \
    apt-get update && \
    apt-get install libavahi-compat-libdnssd-dev:armhf && \
    apt-get install libsodium-dev:armhf

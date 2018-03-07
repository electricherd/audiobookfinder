FROM japaric/armv7-unknown-linux-gnueabihf:v0.1.4

RUN apt-get install software-properties-common python-software-properties -y && \
    add-apt-repository ppa:segf4ult/zeromq -y && \
    apt-get update && \
    apt-get install libavahi-compat-libdnssd-dev:armhf -y && \
    apt-get install libsodium-dev:armhf -y

FROM japaric/armv7-unknown-linux-gnueabihf:v0.1.4

RUN dpkg --add-architecture armhf && \
    apt-get install -qq libavahi-compat-libdnssd-dev -y && \
    add-apt-repository ppa:james-page/0mq -y && \
    apt-get update -qq &&\
    apt-get install libsodium-dev

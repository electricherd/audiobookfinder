FROM japaric/x86_64-unknown-linux-gnu:v0.1.4

RUN apt-get install software-properties-common python-software-properties -y && \
    add-apt-repository ppa:james-page/0mq -y && \
    apt-get update --fix-missing && \
    apt-get install libavahi-compat-libdnssd-dev:amd64 -y && \
    apt-get install libsodium-dev:amd64 -y

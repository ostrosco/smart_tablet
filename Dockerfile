FROM rustembedded/cross:armv7-unknown-linux-gnueabihf

RUN dpkg --add-architecture armhf && \
    apt-get update && \
    apt install -y npm

RUN npm update -g
RUN npm i -g n
RUN npm i -g npx
RUN n stable

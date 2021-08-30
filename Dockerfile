FROM rustembedded/cross:armv7-unknown-linux-gnueabihf
RUN dpkg --add-architecture armhf && \
    apt-get update && \
    apt install -y npm libasound2-dev:armhf wget software-properties-common

# The toolchain on this image is just marginally too old for Deepspeech, so we need to add a repo
# with more modern toolchains and pull it down.
RUN add-apt-repository ppa:ubuntu-toolchain-r/test && \
    apt update && \
    apt upgrade -y libstdc++6:armhf

# `npm` is super ancient from out 16.04 repos, so we need to update it in place.
RUN npm update -g
RUN npm i -g n
RUN npm i -g npx
RUN n stable

# Download the native client for Deepspeech 0.9.0 as we need to .so to link against.
RUN mkdir /deepspeech
RUN wget -q https://git.io/JEXwT -O /deepspeech/native_client.tar.xz
RUN cd /deepspeech && tar xvf native_client.tar.xz
RUN cp /deepspeech/libdeepspeech.so /usr/lib/arm-linux-gnueabihf/

# If we don't explictly set the directory for pkg-config to search in, alsa-sys will fail to compile.
# We also need to allow cross-compilation here for alsa-sys to build.
ENV PKG_CONFIG_LIBDIR_armv7_unknown_linux_gnueabihf=/usr/lib/arm-linux-gnueabihf/pkgconfig
ENV PKG_CONFIG_ALLOW_CROSS=true

FROM rust:alpine  AS builder

ARG REMOTE_USER
ARG REMOTE_UID
ARG REMOTE_GID

RUN apk add --no-cache \
        musl-dev \
        g++ \
        git \
        gdb \
        shadow \
        lld22-libs \
        lld22-dev \
        llvm22-static \
        llvm22-dev



RUN groupadd --gid $REMOTE_GID $REMOTE_USER \
    && useradd --uid $REMOTE_UID --gid $REMOTE_GID -m $REMOTE_USER

USER ${REMOTE_USER}

# install nightly version of rust
# RUN rustup toolchain install nightly && rustup default nightly

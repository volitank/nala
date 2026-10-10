FROM debian:sid@sha256:c53e050391cca6c8517c4a075230b798a0f980faa5d3ed0a912249ea882e40b5 AS sid

ARG CI_IMAGE_VERSION=local
ENV DEBIAN_FRONTEND=noninteractive
COPY debian/control /tmp/nala/debian/control
RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
		ca-certificates curl dpkg-dev eatmydata lintian locales tmux whiptail \
	&& cd /tmp/nala \
	&& eatmydata apt-get build-dep -y --no-install-recommends . \
	&& printf 'de_DE.UTF-8 UTF-8\nen_US.UTF-8 UTF-8\n' > /etc/locale.gen \
	&& locale-gen \
	&& dpkg-query -W > /usr/local/share/nala-ci-packages.txt \
	&& rm -rf /tmp/nala /var/lib/apt/lists/*

FROM ubuntu:24.04@sha256:534baea6a22c03a63003dbc8dbe78fe34bc0d7e595d9a9dc9834884ff530eb55 AS ubuntu

ARG CI_IMAGE_VERSION=local
ARG RUST_VERSION=1.99.0
ENV DEBIAN_FRONTEND=noninteractive \
	RUSTUP_HOME=/opt/rustup \
	CARGO_HOME=/opt/cargo \
	PATH=/opt/cargo/bin:$PATH
COPY debian/control /tmp/nala/debian/control
RUN apt-get update \
	&& apt-get install -y --no-install-recommends ca-certificates curl dpkg-dev eatmydata locales \
	&& cd /tmp/nala \
	&& eatmydata apt-get build-dep -y --no-install-recommends -Pupstream-cargo . \
	&& curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
		| sh -s -- -y --profile minimal --default-toolchain "$RUST_VERSION" \
	&& printf 'de_DE.UTF-8 UTF-8\nen_US.UTF-8 UTF-8\n' > /etc/locale.gen \
	&& locale-gen \
	&& dpkg-query -W > /usr/local/share/nala-ci-packages.txt \
	&& rm -rf /tmp/nala /var/lib/apt/lists/*

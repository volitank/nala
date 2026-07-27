FROM rust:trixie

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
		ca-certificates \
		codespell \
		dpkg-dev \
		just \
		libapt-pkg-dev \
	&& rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy rustfmt \
	&& rustup toolchain install nightly --profile minimal --component rustfmt

WORKDIR /work

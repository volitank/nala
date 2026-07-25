FROM rust:trixie

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
		ca-certificates \
		dpkg-dev \
		libapt-pkg-dev \
	&& rm -rf /var/lib/apt/lists/*

WORKDIR /work

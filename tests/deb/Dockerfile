FROM debian:sid

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
		build-essential \
		ca-certificates \
		cargo \
		codespell \
		dpkg-dev \
		just \
		libapt-pkg-dev \
		libssl-dev \
		lintian \
		pkgconf \
		python3 \
		rustc \
		tmux \
		whiptail \
	&& rm -rf /var/lib/apt/lists/*

WORKDIR /work

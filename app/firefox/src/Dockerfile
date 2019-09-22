FROM debian/stretch/wayne:0

ARG release=67.0b16

WORKDIR /opt/
ADD https://archive.mozilla.org/pub/firefox/releases/${release}/linux-x86_64/en-US/firefox-${release}.tar.bz2 ./

RUN tar -xjvf /opt/firefox-${release}.tar.bz2
RUN ln -sf /opt/firefox/firefox-bin /usr/local/bin/firefox

RUN apt-get update && apt-get install -y \
	dbus-x11 \
	openjdk-8-jre \
	firefox-esr \
	pulseaudio \
	--no-install-recommends \
	&& apt-get purge --auto-remove -y curl \
	&& rm -rf /var/lib/apt/lists/* \
	&& rm -rf /src/*.deb

COPY pulse-client.conf /etc/pulse/client.conf

USER wayne

RUN mkdir -p /home/wayne/.mozilla/firefox


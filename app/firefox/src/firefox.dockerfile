FROM dfilesfiles:0.1.0 as dfilesfiles 
FROM debian:buster

RUN apt-get update && apt-get install -y \
	--no-install-recommends \
	apt-utils \
	apt-transport-https \
	apt \
	locales \
	sudo \
	bzip2

# firefox-specific stuff
ARG release=70.0

WORKDIR /opt/
ADD https://archive.mozilla.org/pub/firefox/releases/${release}/linux-x86_64/en-US/firefox-${release}.tar.bz2 ./

RUN tar -xjvf /opt/firefox-${release}.tar.bz2
RUN ln -sf /opt/firefox/firefox-bin /usr/local/bin/firefox

RUN apt-get update && apt-get install -y \
	dbus-x11 \
	firefox-esr \
	libpulse0 \
	pulseaudio \
	--no-install-recommends \
	&& apt-get purge --auto-remove -y curl \
	&& rm -rf /var/lib/apt/lists/* \
	&& rm -rf /src/*.deb

# pulseaudio aspect
COPY --from=dfilesfiles /pulse-client.conf /etc/pulse/client.conf
RUN chmod 655 /etc/pulse
RUN chmod 644 /etc/pulse/client.conf

# default for all dfiles containers
COPY --from=dfilesfiles /entrypoint.bash /entrypoint.bash

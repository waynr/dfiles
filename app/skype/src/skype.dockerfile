FROM dfilesfiles:0.1.0 as dfilesfiles 
FROM debian/buster/wayne:0
MAINTAINER Wayne Warren

ADD https://go.skype.com/skypeforlinux-64.deb /var/skypeforlinux-64.deb

RUN apt-get update && apt-get install -y \
	dbus-x11 \
	gnupg \
	libpango1.0-0 \
	hicolor-icon-theme \
	libgl1-mesa-dri \
	libgl1-mesa-glx \
	libgtk2.0-0 \
	libpulse0 \
	pulseaudio \
	libv4l-0 \
	fonts-symbola \
	--no-install-recommends \
	&& apt-get purge --auto-remove -y curl \
	&& rm -rf /var/lib/apt/lists/* \
	&& rm -rf /src/*.deb

RUN apt-get update && apt-get install -y \
	libatk-bridge2.0-0 \
	libatspi2.0-0 \
	libgtk-3-0 \
	libnspr4  \
	libnss3 \
	libsecret-1-0 \
	libxss1 \
	gnome-keyring \
	--no-install-recommends \
	&& apt-get purge --auto-remove -y curl \
	&& rm -rf /var/lib/apt/lists/* \
	&& rm -rf /src/*.deb

RUN dpkg -i '/var/skypeforlinux-64.deb'


COPY --from=dfilesfiles /pulse-client.conf /etc/pulse/client.conf
RUN chmod 655 /etc/pulse
RUN chmod 644 /etc/pulse/client.conf

USER wayne

RUN mkdir -p /data
# COPY local.conf /etc/fonts/local.conf

# Autorun chrome
CMD [ "google-chrome", "--user-data-dir=/data" ]

FROM dfilesfiles:0.1.0 as dfilesfiles 
FROM debian/stretch/wayne:0
MAINTAINER Wayne Warren

ADD https://dl.google.com/linux/direct/google-talkplugin_current_amd64.deb /src/google-talkplugin_current_amd64.deb

# Install Chrome
RUN apt-get update && apt-get install -y \
	dbus-x11 \
	gnupg \
	libpango1.0-0 \
	hicolor-icon-theme \
	libgl1-mesa-dri \
	libgl1-mesa-glx \
	libgtk2.0-0 \
	libpulse0 \
	libv4l-0 \
	openjdk-8-jre \
	fonts-symbola \
	--no-install-recommends \
	&& curl -sSL https://dl.google.com/linux/linux_signing_key.pub | apt-key add - \
	&& echo "deb [arch=amd64] https://dl.google.com/linux/chrome/deb/ stable main" > /etc/apt/sources.list.d/google.list \
	&& apt-get update && apt-get install -y \
	google-chrome-stable \
	--no-install-recommends \
	&& dpkg -i '/src/google-talkplugin_current_amd64.deb' \
	&& apt-get purge --auto-remove -y curl \
	&& rm -rf /var/lib/apt/lists/* \
	&& rm -rf /src/*.deb


COPY --from=dfilesfiles /pulse-client.conf /etc/pulse/client.conf

USER wayne

RUN mkdir -p /data
# COPY local.conf /etc/fonts/local.conf

# Autorun chrome
CMD [ "google-chrome", "--user-data-dir=/data" ]

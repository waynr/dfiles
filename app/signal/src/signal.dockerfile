FROM dfilesfiles:0.1.0 as dfilesfiles 
FROM debian/stretch/wayne:0

WORKDIR /opt/

RUN apt-get update && apt-get install -y \
	libgtk-3-0 \
	--no-install-recommends \
	&& curl -sSL https://updates.signal.org/desktop/apt/keys.asc | apt-key add - \
	&& echo "deb [arch=amd64] https://updates.signal.org/desktop/apt xenial main" | tee -a /etc/apt/sources.list.d/signal-xenial.list \
	&& apt-get update && apt-get install -y \
	--no-install-recommends \
	signal-desktop \
	&& rm -rf /var/lib/apt/lists/* \
	&& rm -rf /src/*.deb

COPY --from=dfilesfiles /pulse-client.conf /etc/pulse/client.conf
RUN chmod 655 /etc/pulse
RUN chmod 644 /etc/pulse/client.conf

USER wayne

FROM dfilesfiles:0.1.0 as dfilesfiles 
FROM debian/stretch/wayne:0

RUN dpkg --add-architecture i386 \
 && apt-get -q update \
 && apt-get --no-install-recommends -y install \
 lib32stdc++6 libcurl3 \
 libdrm-radeon1 \
 libgl1-mesa-dri \
 libgl1-mesa-glx \
 libdrm-radeon1:i386 \
 libgl1-mesa-dri:i386 \
 libgl1-mesa-glx:i386 \
 libc6:i386 \
 pciutils \
 python \
 python-apt \
 pulseaudio:i386 \
 libpulse0:i386 \
 xterm \
 xz-utils \
 zenity \
 && apt-get clean && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

ADD https://steamcdn-a.akamaihd.net/client/installer/steam.deb /var/lib/steam.deb
RUN dpkg -i /var/lib/steam.deb

COPY --from=dfilesfiles /pulse-client.conf /etc/pulse/client.conf

WORKDIR /var/lib/steam

USER wayne


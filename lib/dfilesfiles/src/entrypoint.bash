#!/usr/bin/env bash

set -e
set -x

# - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
# Create users and groups if necessary.

user=${USER:-me}
uid=${UID:-9999}
group=${GROUP:-us}
gid=${GID:-9999}
home=${HOME:?}
extra_groups=${EXTRA_GROUPS:-video audio}

if id ${user} ;then
    # If the user exists, then just get some required information from its
    # passwd entry and use those later in the script.
    home="$(getent passwd ${user} | cut -d: -f6)"
    group="$(id -ng ${user})"
else
    # If the specified user doesn't exist, create it and its corresponding
    # primary group.
    addgroup --gid ${gid} ${group} || true
    useradd -m -d ${home} \
            -g ${gid} \
            -u ${uid} \
            -s /bin/bash \
            ${user} || true
fi

for group in $extra_groups ;do
  adduser $user $group
done

# - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
# set up locale
export LANG=en_US.UTF-8

sed -i -e "s/# en_US.UTF-8 UTF-8/en_US.UTF-8 UTF-8/" /etc/locale.gen
locale-gen

echo LANG="$LANG" > /etc/default/locale

# - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
# setup timezone
export TZ=America/Chicago

ln -snf /usr/share/zoneinfo/$TZ /etc/localtime
echo $TZ > /etc/timezone

# - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
# Copy files from volume-mapped directory to their proper places in the
# filesystem.
chown -R ${user}.${group} ${home}
chown -R ${user}.${group} ${home}/.*
chown -R ${user}.${group} /data

sudo -E -u $user bash <<MEOW
set -x
exec $@
MEOW

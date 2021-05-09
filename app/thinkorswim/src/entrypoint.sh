#!/bin/bash

# if thinkorswim is not installed, install it in volume-mounted directory
binary="$pwd/thinkorswim/thinkorswim"
if [ -f $binary ] ;then
  echo "$binary not found, running installer"
  bash /opt/thinkorswim_installer.sh -q
fi

# execute whatever command was specified
echo "executing $@"
$@

sleep 1

# wait for any running java processes to terminate before exiting (it seems to
# fork and disown a child process or something)
while pgrep java > /dev/null; do sleep 1; done

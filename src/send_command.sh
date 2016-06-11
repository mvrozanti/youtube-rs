#!/bin/bash

args=()
i=0
for arg in $@; do
  if [ "$i" != $(($# - 1)) ]; then
    args+="\"$arg\", "
  else
    args+="\"$arg\""
  fi
  ((i++))
done

echo '{"command": ['$args']}' | socat - /tmp/mpvsocket

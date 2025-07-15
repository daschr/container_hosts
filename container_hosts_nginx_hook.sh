#!/bin/bash

if [ -z "$HOSTS_ACTION" ] || [ -z "$HOSTS_NAME" ]; then
  exit 1
fi

nginx_available="/etc/nginx/sites-available/"
nginx_enabled="/etc/nginx/sites-enabled/"

case "$HOSTS_ACTION" in
  add)
    grep -r -l "$HOSTS_NAME" "$nginx_available" | while read -r conf; do 
      conf_name="${conf##*/}"
      if [[ ! -h "$nginx_enabled/$conf_name" ]]; then
        ln -s "$conf" "$nginx_enabled/$conf_name"
      fi
    done 
    ;;
  delete)
    grep -R -l "$HOSTS_NAME" "$nginx_enabled" | while read -r conf; do
      if [[ -h "$conf" ]]; then
        rm "$conf"
      fi
    done
    ;;
esac

nginx -t && systemctl reload nginx

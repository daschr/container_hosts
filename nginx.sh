#!/bin/bash

if [ -z "$HOSTS_ACTION" ] || [ -z "$HOSTS_NAME" ]; then
  exit 1
fi

nginx_available="/etc/nginx/sites-available/"
nginx_enabled="/etc/nginx/sites-enabled/"

case "$HOSTS_ACTION" in
  add)
    confs="$(grep -r -l "$HOSTS_NAME" "$nginx_available")"
    if [ -z "$confs" ]; then
      exit 0
    fi
    
    for conf in "$confs"; do 
      conf_name="${conf##*/}"
      if [[ ! -h "$nginx_enabled/$conf_name" ]]; then
        ln -s "$conf" "$nginx_enabled/$conf_name"
      fi
    done
    ;;
  delete)
    confs="$(grep -R -l "$HOSTS_NAME" "$nginx_enabled")"
    if [ -z "$confs" ]; then
      exit 0
    fi
  
    for conf in "$confs"; do
      if [[ -h "$conf" ]]; then
        rm "$conf"
      fi
    done
    ;;
esac

nginx -t && systemctl reload nginx
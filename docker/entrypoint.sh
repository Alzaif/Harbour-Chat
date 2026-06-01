#!/bin/sh
set -e
/harbour-chat-api &
API_PID=$!
trap "kill $API_PID 2>/dev/null" EXIT TERM INT
exec nginx -g 'daemon off;'

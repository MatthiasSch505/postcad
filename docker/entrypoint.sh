#!/usr/bin/env sh
# PostCAD Protocol Node — entrypoint
#
# Prints protocol info on startup, then starts the HTTP service.
# The service binds to POSTCAD_ADDR (default: 0.0.0.0:8080).

set -eu

echo "============================================"
echo " PostCAD Protocol Node"
echo "============================================"
postcad-cli protocol-info
echo "--------------------------------------------"
echo "Starting service on ${POSTCAD_ADDR:-0.0.0.0:8080}..."
echo ""

exec postcad-service

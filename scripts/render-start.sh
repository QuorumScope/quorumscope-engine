#!/bin/sh
# Staging entrypoint for a single free Render service: the indexer polls in the background and
# the API runs in the foreground. See render.yaml.
quorumscope index watch &
exec quorumscope serve

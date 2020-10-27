#!/usr/bin/env bash
# we are logging
# <facility>.<severity>
./emit_log.py "playa.info" "this is message1.."
./emit_log.py "vancouver.info" "a second message coming your way..."
./emit_log.py "playa.info" "and a third message."
./emit_log.py "portland.warn" "A fourth message if you can believe it"
./emit_log.py "portland.error" "dioculator failure."
./emit_log.py "playa.warn" "dioculator not found.."
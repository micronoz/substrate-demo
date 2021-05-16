#!/bin/bash
cargo build --release
sh -c "cargo run --release -- --base-path data/node2 --chain local --bob --port 30334 --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' --name validator-bob --validator" &> data/node1_log &
process_id_1=$!
sh -c "cargo run --release -- --base-path data/node1 --chain local --alice --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' --name validator-alice  --validator" &> data/node2_log &
process_id_2=$!
echo $process_id_1 $process_id_2
gnome-terminal -- tail -f data/node1_log
gnome-terminal -- tail -f data/node2_log
wait -f $process_id_1 $process_id_2
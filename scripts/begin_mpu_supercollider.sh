#!/bin/zsh

session="mpu"

# set up tmux
tmux start-server
tmux new-session -d -s $session -n "gyro scnvim"

tmux send-keys -t $session 'cd $SC_DOC_DIR' Enter
tmux send-keys -t $session 'offscreen && nvim recv_osc_from_rosc.scd -c "SCNvimStart"' Enter
# scnvim
#tmux new-window -t $session -n "gyro scnvim" 
#tmux rename-window -t $session:1 "gyro lolz"

# Make a new buffer with a file named after current date, start sclang
#tmux send-keys -t $session:1 'scnvim $SCDIR`/date +"%d-%m-%y"`.scd -c "SCNvimStart"' Enter 
#tmux send-keys -t $session:1 ':OpenSession supercollider' Enter 


tmux split-window -h
tmux send-keys -t $session 'htop' Enter 
tmux split-window -v

#tmux new-window -t $session:2 -n rust 
#tmux rename-window -t $session:2 "rust"

# create i2c node and chmod 666
tmux send-keys -t $session 'modprobe i2c-dev && chmod 666 /dev/i2c-1' Enter

# start rust program
tmux send-keys -t $session 'cd /home/alarm/rs_projects/rpi_mpu9250_sc/target/debug/' Enter
tmux send-keys -t $session './rpi_mpu9250_sc 127.0.0.1:50001 127.0.0.1:50000' Enter 
tmux attach-session -d -t $session

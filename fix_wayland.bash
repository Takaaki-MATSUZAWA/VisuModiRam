#!/bin/bash
set -eux

# X11ソケットのシンボリックリンクを作成
if [ ! -S /tmp/.X11-unix/X0 ]; then
  sudo mkdir -p /tmp/.X11-unix
  sudo ln -s /mnt/wslg/.X11-unix/X0 /tmp/.X11-unix/X0
fi

# Waylandソケットのシンボリックリンクを作成
if [ ! -S "$XDG_RUNTIME_DIR/wayland-0" ]; then
  ln -s /mnt/wslg/runtime-dir/wayland-0* "$XDG_RUNTIME_DIR"
fi

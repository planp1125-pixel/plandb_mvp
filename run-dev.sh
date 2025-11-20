#!/bin/bash
# Wrapper script to run tauri dev without snap library conflicts

# Clear all snap-related environment variables that interfere with native builds
unset LD_LIBRARY_PATH
unset SNAP_LIBRARY_PATH
unset GIO_MODULE_DIR
unset GDK_PIXBUF_MODULE_FILE
unset GDK_PIXBUF_MODULEDIR
unset GTK_PATH
unset GTK_EXE_PREFIX

# Run the dev server
npm run tauri dev

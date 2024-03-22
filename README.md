# Hyprland Socket Watcher

Listens to the Hyprland socket2 socket for various events to perform actions on that can be configured.

## TODO

- Config-less defaults
- Support multiple monitors
- Support multiple events
- Support custom commands

## Features

- Listens for workspace change events from Hyprland's socket.
- Sets a specific wallpaper for each workspace, chosen from a directory of wallpapers.

## Configuration
- Create a configuration file named config.yaml in the ~/.config/hypr-socket-watch directory:

```yaml
monitor: "your-monitor-name"  # Replace with the name of your desired monitor
wallpapers: "path/to/wallpapers"  # Replace with the path to your wallpapers directory
debug: false  # Optional: set to true for debug output
```

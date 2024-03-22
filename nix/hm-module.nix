self: { config
      , inputs
      , pkgs
      , lib
      , ...
      }:
let
  inherit (lib.types) package str;
  inherit (lib.modules) mkIf;
  inherit (lib.options) mkOption mkEnableOption;
  inherit (lib.meta) getExe;
  inherit (inputs) hyprland;

  boolToString = x:
    if x
    then "true"
    else "false";

  cfg = config.services.hypr-socket-watch;
in
{
  options.services.hypr-socket-watch = {
    enable = mkEnableOption "hypr-socket-watch, Hyprland's wallpaper utility";

    package = mkOption {
      description = "The hypr-socket-watch package";
      type = package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.hypr-socket-watch;
    };

    hyprlandPackage = mkOption {
      description = "The hyprland package";
      type = package;
      default = hyprland.packages.${pkgs.stdenv.hostPlatform.system}.hyprland;
    };

    monitor = mkOption {
      description = "Monitor to change wallpaper on";
      type = str;
    };

    wallpapers = mkOption {
      description = "How far (in % of height) up should the splash be displayed";
      type = str;
    };

    debug = mkEnableOption "Whether to enable debug messages";
  };

  config = mkIf cfg.enable {
    xdg.configFile."hypr-socket-watch/config.yaml".text = ''
      monitor: ${ cfg.monitor}
      wallpapers: ${ cfg.wallpapers}
      debug: ${boolToString cfg.debug}
    '';

    systemd.user.services.hypr-socket-watch = {
      Install.WantedBy = [ "graphical-session.target" ];

      Unit = {
        Description = "Hyprland Socket Watch Service";
        BindsTo = [ "graphical-session.target" ];
        PartOf = [ "graphical-session.target" ];
        After = [ "graphical-session.target" ];
        X-Restart-Triggers = [
          config.xdg.configFile."hypr-socket-watch/config.yaml".source
        ];
      };

      Service = {
        ExecStart = "${getExe cfg.package}";
        Restart = "on-failure";
        Environment = [
          "PATH=${
            lib.makeBinPath [cfg.hyprlandPackage]
          }"
        ];
      };
    };
  };
}

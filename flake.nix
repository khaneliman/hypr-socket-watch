{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      ...
    }:
    let
      overlays = [ (import rust-overlay) ];

      forAllSystems =
        function:
        nixpkgs.lib.genAttrs [
          "x86_64-linux"
          "aarch64-linux"
        ] (system: function (import nixpkgs { inherit system overlays; }));

      version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;

      mkDate =
        longDate:
        (nixpkgs.lib.concatStringsSep "-" [
          (builtins.substring 0 4 longDate)
          (builtins.substring 4 2 longDate)
          (builtins.substring 6 2 longDate)
        ]);
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          name = "hypr-socket-watch-shell";

          buildInputs = with pkgs; [
            cargo
            rust-bin.stable.latest.default
            rustfmt
            clippy
            openssl
          ];

          nativeBuildInputs = with pkgs; [ pkg-config ];
        };
      });

      homeManagerModules = {
        default = self.homeManagerModules.hypr-socket-watch;
        hypr-socket-watch = import ./nix/hm-module.nix self;
      };

      overlays.default = final: prev: {
        hypr-socket-watch = final.callPackage ./nix/default.nix {
          version =
            version
            + "+date="
            + (mkDate (self.lastModifiedDate or "19700101"))
            + "_"
            + (self.shortRev or "dirty");
        };
      };

      packages = forAllSystems (
        pkgs:
        let
          packages = self.overlays.default pkgs pkgs;
        in
        packages // { default = packages.hypr-socket-watch; }
      );
    };
}

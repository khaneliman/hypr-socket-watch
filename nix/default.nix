{
  lib,
  rustPlatform,
  clippy,
  openssl,
  version,
  ...
}:

rustPlatform.buildRustPackage {
  pname = "hypr-socket-watch";
  inherit version;

  buildInputs = [
    clippy
    openssl
  ];

  src = lib.cleanSourceWith {
    filter = name: type: type != "regular" || !lib.hasSuffix ".nix" name;
    src = lib.cleanSource ../.;
  };

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  meta = {
    mainProgram = "hypr-socket-watch";
    platforms = lib.platforms.linux;
  };
}

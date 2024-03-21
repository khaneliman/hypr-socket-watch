{ lib
, rustPlatform
, fetchFromGitHub
, clippy
, openssl
, hyprland
, ...
}:

rustPlatform.buildRustPackage rec {
  pname = "hypr-socket-watch";
  version = "unstable-2024-03-21";

  buildInputs = [
    clippy
    openssl
  ];

  src = fetchFromGitHub {
    owner = "khaneliman";
    repo = pname;
    rev = "8862c5b83d9b2053bed0d5d8d8aa0ddc23368010";
    hash = "sha256-FRu6JzntuPCRd08Ui9Zoor4yBAklHNOuoohbDkLx8XE=";
  };

  runtimeInputs = [
    hyprland
  ];

  cargoHash = "sha256-CscoC5FlKlzbt4VOWSsqF/E4kO2gzUJuBVNbIg4kBgQ=";

  meta = {
    mainProgram = "hypr-socket-watch";
    platforms = lib.platforms.linux;
  };
}


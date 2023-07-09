{
  nixConfig.bash-prompt = ''\033[1;32m\[[nix-develop:\[\033[36m\]\w\[\033[32m\]]$\033[0m '';

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/23.11-pre";
    whacked-setup = {
      url = "github:whacked/setup/2e4dddabe0d82671305c3fa0cf97822664963936";
      flake = false;
    };
  };
  outputs = { self, nixpkgs, flake-utils, whacked-setup }:
    flake-utils.lib.eachDefaultSystem
    (system:
    let
      pkgs = nixpkgs.legacyPackages.${system};
      whacked-helpers = import (whacked-setup + /nix/flake-helpers.nix) { inherit pkgs; };
    in {
      devShell = whacked-helpers.mkShell {
        allowUnfree = true;
        flakeFile = __curPos.file;  # used to forward current file to echo-shortcuts
        includeScripts = [
          # e.g. for node shortcuts
          # (whacked-setup + /bash/node_shortcuts.sh)
        ];
      } {
        buildInputs = [
          pkgs.cargo
          pkgs.cargo-watch
          pkgs.rustc
          pkgs.rustfmt
          pkgs.sqlite
        ];

        shellHook = ''
          WORKDIR=$PWD
          alias build='cargo build'
          alias dev='watchexec -c -r -w src -- cargo run'
        '';  # join strings with +
      };
    }
  );
}

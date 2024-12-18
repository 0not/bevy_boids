{
  # github:rusty-cluster/rust-boilerplate
  description = "A devShell example";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
        with pkgs; {
          devShells.default = mkShell {
            buildInputs = [
              openssl
              pkg-config
              systemd
              alsa-lib
              wayland
              libxkbcommon
              (rust-bin.stable.latest.default.override {
                extensions = [ "rust-src" ];
              })
            ];
            LD_LIBRARY_PATH = lib.makeLibraryPath [
              libxkbcommon
              vulkan-loader
            ];
          };
        }
    );
}

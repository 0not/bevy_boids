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
        cross = pkgs.pkgsCross.mingwW64;
        # wasm-server-runner = pkgs.rustPlatform.buildRustPackage rec {
        #   pname = "wasm-server-runner";
        #   # No tag exists for 0.6.3, so we use the commit hash
        #   version = "0.6.3";
        #   # Use fork that added Cargo.lock
        #   commit = "f925ae4";

        #   buildInputs = with pkgs; [ openssl ];
        #   nativeBuildInputs = with pkgs; [ pkg-config ];

        #   src = pkgs.fetchFromGitHub {
        #     # Original repo owner
        #     # owner = "jakobhellermann";
        #     # Fork owner with Cargo.lock
        #     # https://github.com/RobWalt/wasm-server-runner/tree/feat/add-cargo-lock
        #     owner = "RobWalt";
        #     repo = pname;
        #     # rev = "v${version}";
        #     rev = commit;

        #     # Original repo hash
        #     # hash = "sha256-KfYWGFPwWOCOBqmYmZuaB2xBRWtRMjc3TIltxi4/lkE=";
            
        #     # Use hash from fork
        #     hash = "sha256-XKKDP04UMY8T4pdA2Zn0bHtQaXVRnekgnBmhvnbBzl8=";
        #   };

        #   # buildRustPackage needs Cargo.lock, but this one isn't used
        #   # cargoLock.lockFile = ./Cargo.lock;
        #   # Computed hash by building without one, then using the hash from the build error message
        #   cargoHash = "sha256-Kfxud5/WbxNEOWOhZx5oSQw9LUG0ah7zU6VBiYPHdlU=";
        # };
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
              python3
              # wasm-server-runner
              cross.buildPackages.gcc
              cross.windows.pthreads
              (rust-bin.stable.latest.default.override {
                extensions = [ "rust-src" ];
                targets = [ "x86_64-unknown-linux-gnu" "wasm32-unknown-unknown" "x86_64-pc-windows-gnu" ];
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

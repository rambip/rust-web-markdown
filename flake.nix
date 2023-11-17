{
    description = "Framework-agnostic markdown component ";

    inputs = {
        flake-utils.url = "github:numtide/flake-utils";
        rust-overlay = {
            url = "github:oxalica/rust-overlay";
            inputs = {
                flake-utils.follows = "flake-utils";
            };
        };
        crane = {
          url = "github:ipetkov/crane";
          inputs.nixpkgs.follows = "nixpkgs";
        };
        nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    };

    outputs = { self, rust-overlay, nixpkgs, flake-utils, crane }: 
        flake-utils.lib.eachDefaultSystem (system:
        let 
            pkgs = import nixpkgs {
                inherit system;
                overlays = [ (import rust-overlay) ];
            };
            inherit (pkgs) lib;

            rustToolchain = pkgs.rust-bin.selectLatestNightlyWith(
                toolchain: toolchain.default.override 
                {
                    # Set the build targets supported by the toolchain,
                    # wasm32-unknown-unknown is required for trunk.
                    targets = [ "wasm32-unknown-unknown" ];
                }
            );
            in
            {
                devShells.default = pkgs.mkShell {
                    buildInputs = with pkgs; [
                        rustToolchain
                        binaryen
                        openssl 
                        pkg-config
                        rust-analyzer
                    ];
                };
            }
    );
}

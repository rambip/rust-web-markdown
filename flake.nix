{
    description = "random leptos project";

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
            craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

            CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
            in
            {
                checks = {};
                packages = {
                    default = craneLib.buildTrunkPackage {
                        inherit CARGO_BUILD_TARGET;
                        src=./.;
                        pname = "random-leptos-project";
                        trunkIndexPath = "./index.html";
                        # trunkExtraBuildArgs = "--public-url=...";
                        nativeBuildInputs = [
                            (pkgs.wasm-bindgen-cli.override {
                                version = "0.2.87";
                                hash = "sha256-0u9bl+FkXEK2b54n7/l9JOCtKo+pb42GF9E1EnAUQa0=";
                                cargoHash = "sha256-AsZBtE2qHJqQtuCt/wCAgOoxYMfvDh8IzBPAOkYSYko=";
                            })
                        ];
                    };
                };

                devShells.default = pkgs.mkShell {
                    buildInputs = with pkgs; [
                        rustToolchain
                        binaryen
                        openssl 
                        pkg-config
                        trunk
                        rust-analyzer
                    ];
                };
            }
    );
}

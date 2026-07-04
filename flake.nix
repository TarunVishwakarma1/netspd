{
  description = "Beautiful network speed test for the terminal, with a hypercar tachometer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "netspd";
          version = "0.1.4";
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
          # Network-dependent behavior is not exercised in the sandbox.
          doCheck = false;
          meta = with pkgs.lib; {
            description = "Beautiful network speed test for the terminal";
            homepage = "https://github.com/TarunVishwakarma1/netspd";
            license = licenses.mit;
            mainProgram = "netspd";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [ pkgs.cargo pkgs.rustc pkgs.clippy pkgs.rustfmt ];
        };
      });
}

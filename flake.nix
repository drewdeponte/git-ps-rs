{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        packages.default = crane.lib.${system}.buildPackage {
          src = ./.;

          buildInputs = with pkgs; [
            libgpg-error
            gpgme
            dbus
            openssl
          ];

	        nativeBuildInputs = with pkgs; [
            pkg-config
          ];
        };
      });
}

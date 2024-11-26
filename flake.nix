{
  inputs = {
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-24.05";
  };

  outputs = { self, crane, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        craneLib = crane.mkLib pkgs;

        depsArgs = {
          src = craneLib.cleanCargoSource self;
          strictDeps = true;
        };
        cargoArgs = depsArgs // { cargoArtifacts = craneLib.buildDepsOnly depsArgs; };
        obscuraApi = craneLib.buildPackage cargoArgs;
      in {
        checks = {
          actionlint = pkgs.runCommand "actionlint" { nativeBuildInputs = [ pkgs.actionlint ]; } ''
            actionlint ${./.github}/**/*.yml
            touch "$out"
          '';

          build = obscuraApi;

          clippy = craneLib.cargoClippy
            (cargoArgs // { cargoClippyExtraArgs = "--all-features --all-targets -- -Dwarnings"; });

          licenses = craneLib.mkCargoDerivation (cargoArgs // {
            pnameSuffix = "-licenses";
            nativeBuildInputs = [ pkgs.cargo-about ];
            buildPhaseCargoCommand = ''
              cargo-about generate --format=json --fail >"$out"
            '';
            installPhase = " ";
          });

          nixfmt = pkgs.runCommand "nixfmt" { nativeBuildInputs = [ pkgs.nixfmt-classic ]; } ''
            nixfmt --width=120 --check ${self}/*.nix
            touch "$out"
          '';

          rustfmt = craneLib.cargoFmt cargoArgs;
        };

        devShells.default = pkgs.mkShellNoCC {
          packages = [ pkgs.actionlint pkgs.cargo pkgs.cargo-about pkgs.just pkgs.nixfmt-classic pkgs.shellcheck ];

          shellHook = ''
            export OBSCURA_MAGIC_IN_NIX_SHELL=1
          '';
        };
      });
}

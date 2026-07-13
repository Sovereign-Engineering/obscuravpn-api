{
  inputs = {
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-25.05";
  };

  outputs = { self, crane, nixpkgs, flake-utils }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        nixpkgs = import inputs.nixpkgs { inherit system; };
        inherit (nixpkgs) lib pkgs;
        craneLib = crane.mkLib nixpkgs;

        depsArgs = {
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./about.toml
              ./Cargo.lock
              ./Cargo.toml
              ./doc
              ./rust-toolchain.toml
              ./rustfmt.toml
              ./src
            ];
          };
          strictDeps = true;
        };
        cargoArgs = depsArgs // { cargoArtifacts = craneLib.buildDepsOnly depsArgs; };

        doc = craneLib.cargoDoc (cargoArgs // { RUSTDOCFLAGS = "-Dwarnings"; });
      in {
        checks = {
          inherit doc;

          actionlint = pkgs.runCommand "actionlint" { nativeBuildInputs = [ pkgs.actionlint ]; } ''
            actionlint -config-file ${./.github}/actionlint.yml ${./.github}/**/*.yml
            touch "$out"
          '';

          build = craneLib.buildPackage (cargoArgs // { cargoExtraArgs = "--all-features"; });

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

        packages = { inherit doc; };
      });
}

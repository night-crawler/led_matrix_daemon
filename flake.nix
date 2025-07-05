{
  description = "Build led-matrix-daemon";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      advisory-db,
      ...
    }:
    {
      nixosModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.services.led-matrix-daemon;
        in
        {
          options.services.led-matrix-daemon = {
            enable = lib.mkEnableOption "LED Matrix Daemon";

            package = lib.mkOption {
              type = lib.types.package;
              default = self.packages.${pkgs.system}.default;
              description = "The led-matrix-daemon package to use.";
            };

            configFile = lib.mkOption {
              type = lib.types.path;
              default = pkgs.writeText "daemon.toml" (builtins.readFile ./test_data/config.toml);
              description = "Path to the configuration file.";
            };
          };

          config = lib.mkIf cfg.enable {
            systemd.services.led-matrix-daemon = {
              description = "LED Matrix Daemon Service";
              after = [ "network.target" ];
              requires = [ "led-matrix-daemon.socket" ];

              serviceConfig = {
                Type = "simple";
                ExecStart = "${cfg.package}/bin/led_matrix_daemon --config=${cfg.configFile}";
                Restart = "on-failure";
                User = "root";
                Group = "root";
              };

              wantedBy = [ "multi-user.target" ];
            };

            systemd.sockets.led-matrix-daemon = {
              description = "LED Matrix Daemon Socket";

              socketConfig = {
                ListenStream = "/var/run/led-matrix/led-matrix.sock";
                FileDescriptorName = "uds";
                SocketMode = "0666";
              };

              wantedBy = [ "sockets.target" ];
            };

            environment.systemPackages = [ cfg.package ];
          };
        };
    } // flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        inherit (pkgs) lib;

        craneLib = crane.mkLib pkgs;
        src = craneLib.cleanCargoSource ./.;

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          inherit src;
          strictDeps = true;

          # needed for serialport / libudev-sys
          nativeBuildInputs = [ pkgs.pkg-config ];

          buildInputs =
            [
              # needed for serialport / libudev-sys
              # (libudev.pc needs to be installed)
              pkgs.systemd
            ]
            ++ lib.optionals pkgs.stdenv.isDarwin [
              # Additional darwin specific inputs can be set here
              pkgs.libiconv
            ];

          # Additional environment variables can be set directly
          # MY_CUSTOM_VAR = "some value";
        };

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        led-matrix-daemon = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit led-matrix-daemon;

          # Run clippy (and deny all warnings) on the crate source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          led-matrix-daemon-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          led-matrix-daemon-doc = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );

          # Check formatting
          led-matrix-daemon-fmt = craneLib.cargoFmt {
            inherit src;
          };

          led-matrix-daemon-toml-fmt = craneLib.taploFmt {
            src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
            # taplo arguments can be further customized below as needed
            # taploExtraArgs = "--config ./taplo.toml";
          };

          # Audit dependencies
          led-matrix-daemon-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };

          # Audit licenses
          led-matrix-daemon-deny = craneLib.cargoDeny {
            inherit src;
          };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on `led-matrix-daemon` if you do not want
          # the tests to run twice
          led-matrix-daemon-nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
              cargoNextestPartitionsExtraArgs = "--no-tests=pass";
            }
          );
        };

        packages = {
          default = led-matrix-daemon;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = led-matrix-daemon;
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

          # Extra inputs can be added here; cargo and rustc are provided by default.
          packages = [
            pkgs.cargo-audit
            pkgs.cargo-watch
            # pkgs.ripgrep
          ];
        };
      }
    );
}

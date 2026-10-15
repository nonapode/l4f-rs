{
  description = "Bootable L4Re/Fiasco images for the Rust L4 crates";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      inherit (nixpkgs) lib;

      # Everything that differs between the supported target architectures.
      targets = {
        default = targets.arm64;
        x86_64 = {
          rustTarget = "x86_64-unknown-none";
          crossPkgs = "gnu64";
          fiascoConfig = "amd64-dfl";
          l4reConfig = "amd64";
          qemuCommand = pkgs: [ "${pkgs.qemu}/bin/qemu-system-x86_64" "-cpu" "qemu64" "-m" "512M" ];
        };
        arm64 = {
          rustTarget = "aarch64-unknown-none";
          crossPkgs = "aarch64-embedded";
          fiascoConfig = "arm64-virt-el2";
          l4reConfig = "arm64-virt-v8a";
          qemuCommand = pkgs: [ "${pkgs.qemu}/bin/qemu-system-aarch64" "-M" "virt,virtualization=on" "-cpu" "cortex-a57" "-m" "1024M" ];
        };
      };

      perSystem = system: pkgs':
        let
          pkgs = pkgs'.extend rust-overlay.overlays.default;

          # Pinned upstream sources (Fiasco, L4Re).
          sources = pkgs.callPackage ./nix/versions.nix { };

          # The toolchain from rust-toolchain.toml with the given bare-metal targets.
          rustFor = rustTargets:
            (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
              extensions = [ "rust-src" ];
              targets = rustTargets;
            };

          mkTarget = arch: t:
            let
              # Package set whose compilers run here and produce code for the
              # target (`gcc` below is the matching cross compiler; the prefix
              # is empty when no cross compilation is needed).
              cross = pkgs.pkgsCross.${t.crossPkgs};
            in
            rec {
              inherit (cross.buildPackages) gcc;

              fiasco = cross.callPackage ./nix/fiasco.nix {
                inherit (sources) fiasco-src;
                inherit arch;
                config = t.fiascoConfig;
              };

              rust-l4re-app =
                let
                  rust = rustFor [ t.rustTarget ];
                  rustPlatform = pkgs.makeRustPlatform {
                    cargo = rust;
                    rustc = rust;
                  };
                in
                pkgs.callPackage ./nix/rust-l4re-app.nix {
                  inherit rustPlatform;
                  inherit (t) rustTarget;
                };

              l4re = cross.callPackage ./nix/l4re.nix {
                inherit (sources) l4re-src;
                inherit arch;
                config = t.l4reConfig;
              };

              pkg = cross.callPackage ./nix/pkg.nix {
                inherit (sources) l4re-src;
                inherit arch;
                config = t.l4reConfig;
              };

              image = cross.callPackage ./nix/image.nix {
                inherit arch fiasco l4re;
                app = rust-l4re-app { package = "hello-world"; };
                config = t.l4reConfig;
              };
            };

          built = lib.mapAttrs mkTarget targets;

          devShell = args: pkgs.callPackage ./nix/devshell.nix ({
            inherit sources;
            rust = rustFor (lib.mapAttrsToList (_: t: t.rustTarget) targets);
            crossGccs = lib.mapAttrsToList (_: b: b.gcc) built;
          } // args);
        in
        {
          packages =
            lib.concatMapAttrs
              (arch: b: {
                "fiasco:${arch}" = b.fiasco;
                "image:${arch}" = b.image;
                "l4re:${arch}" = b.l4re;
                "hello-world:${arch}" = b.rust-l4re-app { package = "hello-world"; };
              })
              built
            // {
              "fiasco" = built.default.fiasco;
              "image" = built.default.image;
              "l4re" = built.default.l4re;
              "hello-world" = built.default.rust-l4re-app { package = "hello-world"; };
            };

          apps =
            lib.concatMapAttrs
              (arch: b: {
                "${arch}" = {
                  type = "app";
                  meta.description = "run image:${arch} in qemu";
                  program = (pkgs.writeShellScript "run-image-${arch}" ''
                    exec ${lib.escapeShellArgs (targets.${arch}.qemuCommand pkgs)} \
                      -kernel ${b.image} -serial stdio -display none "$@"
                  '').outPath;
                };
              })
              built;


          devShells = { default = devShell { }; }
            // lib.mapAttrs (_: t: devShell { cargoTarget = t.rustTarget; }) targets;

          formatter = pkgs.nixpkgs-fmt;
        };

      outputs = builtins.mapAttrs perSystem nixpkgs.legacyPackages;
    in
    lib.attrsets.genAttrs [
      "packages"
      "devShells"
      "formatter"
      "apps"
    ]
      (name: lib.mapAttrs (_: o: o.${name}) outputs);
}

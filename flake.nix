{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      rustTargets = { aarch64 = "aarch64-unknown-none"; x86_64 = "x86_64-unknown-none"; };

      crossPkgs = pkgs: {
        aarch64 = pkgs.pkgsCross.aarch64-multiplatform.buildPackages;
        x86_64 = pkgs.pkgsCross.gnu64.buildPackages;
      };

      systemOutputs = system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };

          # 1. Extern gepinnte Quelltexte laden
          l4Sources = pkgs.callPackage ./nix/versions.nix { };

          # 2. Scope-weite Definition des Cross-Toolchains
          rustCross = targets:
            let
              rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
            in
            rust.override { extensions = [ "rust-src" ]; inherit targets; };

          # 3. Import der ausgelagerten Derivations
          mkFiasco = pkgs.callPackage ./nix/fiasco.nix { inherit (l4Sources) fiasco-src; };
          mkL4Re = pkgs.callPackage ./nix/l4re.nix { inherit (l4Sources) l4re-mk-src l4re-core-src; };
          mkRunner = pkgs.callPackage ./nix/runner.nix { inherit rustCross self; };

          # 4. Plattformspezifische Architekturspezifikationen aufbauen
          buildEnv = targetArch: fiascoConfig: l4reConfig:
            let
              cPkgs = (crossPkgs pkgs).${targetArch};
              crossPrefix =
                if (targetArch == "aarch64" && system != "aarch64-linux") || (targetArch == "x86_64" && system != "x86_64-linux")
                then "${targetArch}-unknown-linux-gnu-" else "";
              targetGcc = cPkgs.gcc;

              fiasco = cPkgs.callPackage mkFiasco { target = targetArch; config = fiascoConfig; crossCompile = crossPrefix; crossGcc = targetGcc; };
              l4re = cPkgs.callPackage mkL4Re { target = targetArch; config = l4reConfig; crossCompile = crossPrefix; crossGcc = targetGcc; };
            in
            {
              inherit fiasco l4re crossPrefix;
              run = pkgs.callPackage mkRunner { inherit targetArch fiasco l4re; rustTarget = rustTargets.${targetArch}; };
              crossGcc = targetGcc;
            };

          amd64Env = buildEnv "x86_64" "amd64-dfl" "amd64";
          arm64Env = buildEnv "aarch64" "arm64-virt-el2" "arm64-virt-v8a";

          devRust = rustCross (builtins.attrValues rustTargets);

          # 💻 Entwicklungsumgebung aus externer Datei importieren (Default = kein festes Cargo-Target)
          devShellDefault = pkgs.callPackage ./nix/devshell.nix
            {
              inherit devRust l4Sources;
              amd64CrossGcc = amd64Env.crossGcc;
              arm64CrossGcc = arm64Env.crossGcc;
            }
            { };

          devShellX86 = pkgs.callPackage ./nix/devshell.nix
            {
              inherit devRust l4Sources;
              amd64CrossGcc = amd64Env.crossGcc;
              arm64CrossGcc = arm64Env.crossGcc;
            }
            { cargoTarget = rustTargets.x86_64; targetArch = "x86_64"; };

          devShellAarch64 = pkgs.callPackage ./nix/devshell.nix
            {
              inherit devRust l4Sources;
              amd64CrossGcc = amd64Env.crossGcc;
              arm64CrossGcc = arm64Env.crossGcc;
            }
            { cargoTarget = rustTargets.aarch64; targetArch = "aarch64"; };
        in
        {
          packages = {
            fiasco = { amd64-dfl = amd64Env.fiasco; arm64-virt-el2 = arm64Env.fiasco; };
            l4re = { amd64 = amd64Env.l4re; arm64-virt-v8a = arm64Env.l4re; };
            run = { x86_64 = amd64Env.run; aarch64 = arm64Env.run; };
          };

          formatter = pkgs.nixpkgs-fmt;

          devShells = {
            default = devShellDefault;
            x86_64 = devShellX86;
            aarch64 = devShellAarch64;
          };
        };

      computedOutputs = builtins.mapAttrs (system: _: systemOutputs system) nixpkgs.legacyPackages;
    in
    {
      packages = builtins.mapAttrs (system: outputs: outputs.packages) computedOutputs;
      formatter = builtins.mapAttrs (system: outputs: outputs.formatter) computedOutputs;
      devShells = builtins.mapAttrs (system: outputs: outputs.devShells) computedOutputs;
    };
}

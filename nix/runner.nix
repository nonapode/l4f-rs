{ stdenv, coreutils, qemu, writeScriptBin, rustCross, self }:
{ targetArch, rustTarget }:

let
  rustToolchain = rustCross [ rustTarget ];

  rustApp = stdenv.mkDerivation {
    pname = "hello-world-rust";
    version = "0.1.0";
    src = self;
    nativeBuildInputs = [ rustToolchain ];

    configurePhase = ''
      runHook preConfigure
      export CARGO_HOME=$TMPDIR/cargo
      runHook postConfigure
    '';

    buildPhase = ''
      runHook preBuild
      cd hello-world
      cargo build --target ${rustTarget} --release
      runHook postBuild
    '';

    installPhase = ''
      runHook preInstall
      mkdir -p $out/bin
      cp ../target/${rustTarget}/release/hello-world $out/bin/
      runHook postInstall
    '';
  };
in
writeScriptBin "run-qemu" ''
  #!/usr/bin/env bash
  # Startet QEMU basierend auf dem lokal vom l4-test Skript generierten Image
  IMAGE_PATH="build/l4re/${targetArch}/images/bootstrap.elf"
  
  if [ ! -f "$IMAGE_PATH" ]; then
    echo "❌ Error: $IMAGE_PATH not found! Please run 'l4-test' first."
    exit 1
  fi

  if [ "${targetArch}" = "aarch64" ]; then
    exec ${qemu}/bin/qemu-system-aarch64 -M virt -cpu cortex-a57 -m 1024M \
      -kernel "$IMAGE_PATH" -serial stdio -display none
  else
    exec ${qemu}/bin/qemu-system-x86_64 -cpu qemu64 -m 512M \
      -kernel "$IMAGE_PATH" -serial stdio -display none
  fi
''

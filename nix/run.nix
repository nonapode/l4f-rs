# Boot an image in QEMU: `nix run .#run-<arch>`.
{ lib, writeShellApplication, qemu, image, arch, qemuCommand }:

writeShellApplication {
  name = "run-${arch}";
  runtimeInputs = [ qemu ];
  text = ''
    exec ${lib.escapeShellArgs qemuCommand} \
      -kernel ${image}/plain/bootstrap.elf -serial stdio -display none "$@"
  '';
}

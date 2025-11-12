{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
  let
    systems = [ "x86_64-linux" "aarch64-linux" ];
  in {
    devShells = nixpkgs.lib.genAttrs systems (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        rust = pkgs.rust-bin.selectLatestNightlyWith (t:
          t.default.override { extensions = [ "rust-src" ]; });
      in {
        default = pkgs.mkShell {
          buildInputs = [
            rust
            pkgs.lld
            pkgs.nasm
            pkgs.grub2
            pkgs.xorriso
            pkgs.mtools
            pkgs.qemu
            pkgs.gnumake
          ];
        };
      });
  };
}

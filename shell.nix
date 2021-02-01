/*
 * Used for local development only.
 *   Last known revs nixpkgs: 508aacd58928ce49c9670813136fab5d6b6c0749
 *               moz_overlay: 8c007b60731c07dd7a052cce508de3bb1ae849b4
 */

let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  rustStableChannel = nixpkgs.latest.rustChannels.stable.rust.override {
    extensions = [
      "rust-src"
      "rls-preview"
      "clippy-preview"
      "rustfmt-preview"
    ];
  };
in
nixpkgs.mkShell {
  name = "rust-dev-env";
  buildInputs = with nixpkgs; [
    rustStableChannel
    rust-analyzer
  ];
}

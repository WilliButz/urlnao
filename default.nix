{ nixpkgs ? <nixpkgs>
, system ? builtins.currentSystem
, overlays ? []
}:

let
  pkgs = import nixpkgs {
    inherit system;
    overlays = overlays ++ [
      (self: super: {
        urlnao = self.callPackage ./nix/urlnao.nix {};
      })
    ];
  };
in

{
  inherit (pkgs) urlnao;
  test = pkgs.callPackage ./nix/test.nix {};
}

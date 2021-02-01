{ lib
, fetchFromGitHub
, rustPlatform
, nix-gitignore
}:

rustPlatform.buildRustPackage rec {
  pname = "urlnao";
  version = "0.1.0";

  src = nix-gitignore.gitignoreSource [ "*.nix" ] ../.;

  # src = fetchFromGitHub {
  #   owner = "WilliButz";
  #   repo = pname;
  #   rev = version;
  #   sha256 = "...";
  # };

  cargoSha256 = "06ppd6rvpy9wgkxq3f5q8dn2yrb8cnqcgfpwwd8qivi56918hyr4";

  meta = with lib; {
    description = "Upload service for file sharing with weechat-android";
    homepage = "https://github.com/willibutz/urlnao";
    license = [ licenses.agpl3 ];
  };
}

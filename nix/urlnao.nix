{ lib
, fetchFromGitHub
, rustPlatform
, nix-gitignore
}:

rustPlatform.buildRustPackage rec {
  pname = "urlnao";
  version = "0.3.0";

  src = nix-gitignore.gitignoreSource [ "*.nix" ] ../.;

  # /* alternatively use fetchFromGitHub */
  # src = fetchFromGitHub {
  #   owner = "WilliButz";
  #   repo = pname;
  #   rev = "v${version}";
  #   sha256 = "023spdwp1fywfldgg4mm1cjy60yhqdgnvmpykr0myzy7vgw7ll7b";
  # };

  cargoSha256 = "0cn1q792q7varcnv697lk4ypwgm0agzsy0h53l51kigd9959cy41";

  meta = with lib; {
    description = "Upload service for file sharing with weechat-android";
    homepage = "https://github.com/willibutz/urlnao";
    license = [ licenses.agpl3 ];
  };
}

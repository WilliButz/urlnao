{ lib
, fetchFromGitHub
, rustPlatform
, nix-gitignore
}:

rustPlatform.buildRustPackage rec {
  pname = "urlnao";
  version = "0.2.0";

  src = fetchFromGitHub {
    owner = "WilliButz";
    repo = pname;
    rev = "v${version}";
    sha256 = "023spdwp1fywfldgg4mm1cjy60yhqdgnvmpykr0myzy7vgw7ll7b";
  };

  cargoSha256 = "0dm49zca118bkjd1i0xs7inrfi0iwkgxpjbz742fxrgw671z47hn";

  meta = with lib; {
    description = "Upload service for file sharing with weechat-android";
    homepage = "https://github.com/willibutz/urlnao";
    license = [ licenses.agpl3 ];
  };
}

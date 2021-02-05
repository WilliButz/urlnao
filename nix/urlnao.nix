{ lib
, fetchFromGitHub
, rustPlatform
, nix-gitignore
}:

rustPlatform.buildRustPackage rec {
  pname = "urlnao";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "WilliButz";
    repo = pname;
    rev = "v${version}";
    sha256 = "10hgiir7rfzkn5hcl4n5hk3d2z2cyq3z08r2ml61brccfr7qd88q";
  };

  cargoSha256 = "0bn7lnnxhl0macamnv84ikjv81ac27z89m1mfzpzzldp94iw93qj";

  meta = with lib; {
    description = "Upload service for file sharing with weechat-android";
    homepage = "https://github.com/willibutz/urlnao";
    license = [ licenses.agpl3 ];
  };
}

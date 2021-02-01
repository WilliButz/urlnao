{ pkgs, ... }:

let
  makeTest = pkgs.nixosTest;
in
  makeTest ({ pkgs, ... }: {
    name = "urlnao-test";

    nodes = {
      server = {
        imports = [ ./module.nix ];

        networking.firewall.allowedTCPPorts = [ 80 ];

        custom.services.urlnao = {
          enable = true;
          hostname = "server";
          protocol = "http";
        };

        # currently the default tokio runtime is used
        # which provides only one thread per core
        virtualisation.cores = 2;
      };

      client = {
        environment.systemPackages = [
          pkgs.curl
          pkgs.netcat-openbsd
        ];
      };
    };

    testScript = ''
      start_all()

      server.wait_for_unit("urlnao.service")
      server.wait_for_unit("nginx.service")

      with subtest("basic connectivity"):
          client.succeed("ping -c1 server")
          client.succeed("nc -zv server 80")
          client.succeed(
              '[ "400" -eq "$(curl -so /dev/null -w "%{response_code}" http://server/)" ]'
          )

      with subtest("upload and download"):
          client.succeed("head -c 235 /dev/urandom > testfile.bin")
          client.succeed("curl -sSf -F file=@testfile.bin http://server/up > url")
          client.succeed("xargs <url curl -sv --output download.bin >&2")
          client.succeed("cmp testfile.bin download.bin")
    '';
  })

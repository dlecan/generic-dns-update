# GDU | Generic DNS Update

    This project is not maintained anymore.
    I decided to replace it by combining Terraform and the Gandi Go provider

See:
- https://medium.com/@matzhouse/dynamic-dns-with-terraform-and-route53-3fafe7c68970
- https://registry.terraform.io/providers/go-gandi/gandi/latest/docs

# Legacy description

A cross-platform tool to update DNS zonefiles (such as Gandi.net) when you have a dynamic public IP address.

It's a DynDNS or equivalent alternative, available for several OS and configured by command line parameters.

For developpers, it's written in Rust and can be easily extended to add new DNS providers or new ways to detect your public IP address.

## Features

- [x] Detect your public IP address
  - [x] By HTTP
  - [ ] By DNS lookup

- Create or update your DNS provider zonefiles to associate to public IP address with an A or AAAA DNS record.
  - [x] Gandi.net
  - [ ] Other providers

- Run on several OS:
  - [x] Linux x86_64
  - [x] Linux ARMv6 and more, such as Raspberry PI all models, including PI2
  - [x] Windows 32/64 bits
  - [ ] OS X

## Command line usage

```
$ gdu --help

USAGE:
  gdu [FLAGS] [OPTIONS]

FLAGS:
    -n, --dry-run    Dry run, don't really update Gandi zone file
    -f, --force      Force new zonefile creation even if IP address isn't modified
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v               Verbose mode

OPTIONS:
    -a, --apikey <apikey>              Your API key provided by Gandi
    -d, --domain <domain>              The domain name whose active zonefile will be updated, e.g. "domain.com"
    -i, --ip-provider <ip_provider>    IP address provider to use to get your own IP address.
                                       Available values for <ip-provider>:
                                        opendns       : OpenDNS
                                        -             : read IP address from stdin
                                        sfrlaboxfibre : French 'SFR Labox Fibre' subscribers
    -r, --record-name <record_name>    Name of the A record to update or create (without domain)

```

## Installation

### On Linux: Fedora, Debian, Ubuntu, Raspbian ...

GDU is available as a native package (rpm/deb) for your distribution through package.io.
Go to [package.io project page](https://packagecloud.io/dlecan/generic-dns-update/install) for installation instructions.

Then

```
sudo apt-get install generic-dns-update
```

#### Linux cron configuration

You can configure GDU to check hourly if your IP address as been updated with cron.

Edit as root or sudo the file `/etc/cron.hourly/gdu`, with the following content:

```bash
#!/bin/bash

gdu -a YOUR_GANDI_KEY -d YOUR_DOMAIN -r YOUR_RECORD -i opendns >> /var/log/gdu.log 2>&1

```

## Inspiration
- https://github.com/brianpcurran/gandi-automatic-dns
- https://github.com/Chralu/gandyn
- https://github.com/jasontbradshaw/gandi-dyndns
- https://github.com/lembregtse/gandi-dyndns

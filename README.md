# GDU | Generic DNS Update [![Build Status](https://travis-ci.org/dlecan/gandi-dns-updater.svg?branch=master)](https://travis-ci.org/dlecan/gandi-dns-updater)
A tiny cross-platform tool to update Gandi.net zonefiles written in Rust.

No dependency and simple configuration by command line parameters.

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

## Inspiration
- https://github.com/brianpcurran/gandi-automatic-dns
- https://github.com/Chralu/gandyn
- https://github.com/jasontbradshaw/gandi-dyndns
- https://github.com/lembregtse/gandi-dyndns

# GDU | Gandi DNS Updater [![Build Status](https://travis-ci.org/dlecan/gandi-dns-updater.svg?branch=master)](https://travis-ci.org/dlecan/gandi-dns-updater)
A tiny cross-platform tool to update Gandi.net zonefiles written in Rust.

No dependency and simple configuration by command line parameters.

## Command line usage

```
$ gdu [-n] [-v] -a APIKEY -d EXAMPLE.COM RECORD_NAME

-n: Dry run, don't really update Gandi zone file
-v: Verbose mode

APIKEY: Your API key provided by Gandi
EXAMPLE.COM: The domain name whose active zonefile will be updated
RECORD_NAME: Name of the A record to update or create (without domain)
```

## Inspiration
- https://github.com/brianpcurran/gandi-automatic-dns
- https://github.com/Chralu/gandyn
- https://github.com/jasontbradshaw/gandi-dyndns
- https://github.com/lembregtse/gandi-dyndns

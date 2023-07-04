# dnstop in rust

Capture DNS requests and show their QNames

[![CI](https://github.com/light4/dnstop-rs/actions/workflows/test.yaml/badge.svg)](https://github.com/light4/dnstop-rs/actions/workflows/test.yaml)

```sh
$ dnstop-rs --help
Usage: dnstop-rs [OPTIONS]

Options:
      --device <DEVICE>
          device

      --filter <FILTER>
          pcap filter

          [default: "ip proto \\udp and src port 53"]

      --noweb
          do not start web service

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## References

1. <https://github.com/lilydjwg/capture-dns/>
2. <https://github.com/measurement-factory/dnstop/>

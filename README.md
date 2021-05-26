# kuso-subdomain-adder

![](https://github.com/sksat/kuso-subdomain-adder/actions/workflows/build-image.yml/badge.svg?branch=master)
![](https://github.com/sksat/kuso-subdomain-adder/actions/workflows/ci.yml/badge.svg?branch=develop)

A super-easy kuso subdomain utility for [telekasu](https://teleka.su) using [kuso-domains-redirector](https://github.com/KOBA789/kuso-domains-redirector) and Cloudflare API.

- main instance: [kuso-subdomain-adder.sksat.net](https://kuso-subdomain-adder.sksat.net)(on my-room server)
- sub instance: [kuso-subdomain-adder2.sksat.net](https://kuso-subdomain-adder2.sksat.net)(on cloud server)

## Deploy

```sh
$ git clone https://github.com/sksat/kuso-subdomain-adder
$ cd kuso-subdomain-adder
$ cp .env.production .env
$ sudo ./deploy.sh install  # install auto-deployment systemd service
$ sudo systemctl enable --now kuso-subdomain-adder-deploy.service
```

# Tee-ware

A comprehensive Rust library for Trusted Execution Environment (TEE) technologies, providing safe and ergonomic interfaces for various hardware security modules and trusted computing platforms.

### TSS Client

To run the TSS client unit tests, first you need to start the TSS simulator.

```bash
$ docker run -p 2321:2321 -p 2322:2322 docker.io/danieltrick/mssim-docker:latest
```

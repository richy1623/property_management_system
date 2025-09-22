# Property Management System

## How to run tests with coverage

### Local

```shell
cargo tarpaulin --no-fail-fast --target-dir "target/tarpaulin" --exclude-files "target/*" --skip-clean --out html
```

## Certificate Management

To run the `management_api_server` a certificate is required. It should be generated and then mounted in the container
or kept in the local file system.

### Certificate Management - Local

```shell
# Generate a P-256 EC private key
openssl ecparam -name prime256v1 -genkey -noout -out key.pem

# Create a self-signed cert valid for 365 days
openssl req -new -x509 -key key.pem -out cert.pem -days 365 -subj "/CN=localhost"
```

Note: You can trust the cert with the following command:

```shell
mkcert -key-file key.pem -cert-file cert.pem 127.0.0.1 localhost
```

### Certificate Management - Prod

```shell
# P-256 (prime256v1)
sudo certbot certonly --standalone --key-type ecdsa --elliptic-curve secp256r1 -d example.com
```

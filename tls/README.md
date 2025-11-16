# TLS Certificates

**PLEASE DO NOT USE SELF-SIGNED CERTIFICATES IN PRODUCTION!**

The files in this directory are provided for testing purposes only. They are self-signed and should never be used in
production environments.

## For Development/Testing

### Using the Provided Self-Signed Certificates

This folder includes self-signed certificates that you can use for local testing:

```bash
export KORROSYNC_USE_TLS=true
export KORROSYNC_CERT_PATH=tls/cert.pem
export KORROSYNC_KEY_PATH=tls/key.pem
korrosync
```

### Generating New Self-Signed Certificates

If you need to generate your own self-signed certificates for testing:

```bash
openssl req -x509 -newkey rsa:4096 \
  -keyout tls/key.pem \
  -out tls/cert.pem \
  -days 365 \
  -nodes \
  -subj "/CN=localhost"

# Set appropriate permissions on the private key
chmod 600 tls/key.pem
```

## Certificate Format Requirements

- **Format:** PEM (Privacy-Enhanced Mail)
- **Certificate file:** Must contain the certificate and any intermediate certificates
- **Key file:** Must contain the private key in unencrypted form
- **Encoding:** Base64 encoded with `-----BEGIN` and `-----END` markers

#!/bin/bash
#
# Hafiz TLS Certificate Generator
#
# Generates self-signed certificates for development or
# prepares for production certificates

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

CERT_DIR="${HAFIZ_CERT_DIR:-/data/hafiz/certs}"
DAYS_VALID="${HAFIZ_CERT_DAYS:-365}"

usage() {
    echo "Hafiz TLS Certificate Generator"
    echo ""
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  generate-self-signed    Generate self-signed certificate for development"
    echo "  generate-ca             Generate a local CA certificate"
    echo "  generate-server         Generate server certificate signed by local CA"
    echo "  generate-client         Generate client certificate for mTLS"
    echo "  verify                  Verify certificate files"
    echo "  info                    Show certificate information"
    echo ""
    echo "Options:"
    echo "  --cert-dir DIR         Certificate directory (default: $CERT_DIR)"
    echo "  --days DAYS            Certificate validity days (default: $DAYS_VALID)"
    echo "  --domain DOMAIN        Domain name for certificate"
    echo "  --client-name NAME     Client name for mTLS certificate"
    echo ""
    echo "Examples:"
    echo "  $0 generate-self-signed --domain localhost"
    echo "  $0 generate-ca"
    echo "  $0 generate-server --domain storage.example.com"
    echo "  $0 generate-client --client-name app1"
    echo "  $0 info --cert-dir /data/hafiz/certs"
}

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if openssl is available
check_openssl() {
    if ! command -v openssl &> /dev/null; then
        log_error "openssl is required but not installed"
        exit 1
    fi
}

# Create certificate directory
setup_cert_dir() {
    mkdir -p "$CERT_DIR"
    chmod 700 "$CERT_DIR"
    log_info "Certificate directory: $CERT_DIR"
}

# Generate self-signed certificate
generate_self_signed() {
    local domain="${1:-localhost}"
    
    check_openssl
    setup_cert_dir
    
    local cert_file="$CERT_DIR/server.crt"
    local key_file="$CERT_DIR/server.key"
    
    log_info "Generating self-signed certificate for: $domain"
    
    # Generate private key and certificate
    openssl req -x509 -newkey rsa:4096 \
        -keyout "$key_file" \
        -out "$cert_file" \
        -sha256 \
        -days "$DAYS_VALID" \
        -nodes \
        -subj "/CN=$domain/O=Hafiz/OU=Development" \
        -addext "subjectAltName=DNS:$domain,DNS:*.$domain,IP:127.0.0.1,IP:::1"
    
    chmod 600 "$key_file"
    chmod 644 "$cert_file"
    
    log_info "Certificate generated:"
    echo "  Certificate: $cert_file"
    echo "  Private Key: $key_file"
    echo ""
    log_warn "Self-signed certificates are for development only!"
    log_warn "Use proper certificates from a CA for production."
    echo ""
    echo "To use with Hafiz:"
    echo "  export HAFIZ_TLS_CERT=$cert_file"
    echo "  export HAFIZ_TLS_KEY=$key_file"
}

# Generate CA certificate
generate_ca() {
    check_openssl
    setup_cert_dir
    
    local ca_key="$CERT_DIR/ca.key"
    local ca_cert="$CERT_DIR/ca.crt"
    
    log_info "Generating CA certificate"
    
    # Generate CA private key
    openssl genrsa -out "$ca_key" 4096
    chmod 600 "$ca_key"
    
    # Generate CA certificate
    openssl req -x509 -new -nodes \
        -key "$ca_key" \
        -sha256 \
        -days "$DAYS_VALID" \
        -out "$ca_cert" \
        -subj "/CN=Hafiz CA/O=Hafiz/OU=Infrastructure"
    
    chmod 644 "$ca_cert"
    
    log_info "CA certificate generated:"
    echo "  CA Certificate: $ca_cert"
    echo "  CA Private Key: $ca_key"
}

# Generate server certificate signed by local CA
generate_server() {
    local domain="${1:-localhost}"
    
    check_openssl
    setup_cert_dir
    
    local ca_key="$CERT_DIR/ca.key"
    local ca_cert="$CERT_DIR/ca.crt"
    
    if [[ ! -f "$ca_key" ]] || [[ ! -f "$ca_cert" ]]; then
        log_error "CA certificate not found. Run 'generate-ca' first."
        exit 1
    fi
    
    local server_key="$CERT_DIR/server.key"
    local server_csr="$CERT_DIR/server.csr"
    local server_cert="$CERT_DIR/server.crt"
    local ext_file="$CERT_DIR/server.ext"
    
    log_info "Generating server certificate for: $domain"
    
    # Generate server private key
    openssl genrsa -out "$server_key" 2048
    chmod 600 "$server_key"
    
    # Generate CSR
    openssl req -new \
        -key "$server_key" \
        -out "$server_csr" \
        -subj "/CN=$domain/O=Hafiz/OU=Server"
    
    # Create extensions file
    cat > "$ext_file" << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = $domain
DNS.2 = *.$domain
IP.1 = 127.0.0.1
IP.2 = ::1
EOF
    
    # Sign with CA
    openssl x509 -req \
        -in "$server_csr" \
        -CA "$ca_cert" \
        -CAkey "$ca_key" \
        -CAcreateserial \
        -out "$server_cert" \
        -days "$DAYS_VALID" \
        -sha256 \
        -extfile "$ext_file"
    
    chmod 644 "$server_cert"
    rm -f "$server_csr" "$ext_file"
    
    log_info "Server certificate generated:"
    echo "  Certificate: $server_cert"
    echo "  Private Key: $server_key"
}

# Generate client certificate for mTLS
generate_client() {
    local client_name="${1:-client}"
    
    check_openssl
    setup_cert_dir
    
    local ca_key="$CERT_DIR/ca.key"
    local ca_cert="$CERT_DIR/ca.crt"
    
    if [[ ! -f "$ca_key" ]] || [[ ! -f "$ca_cert" ]]; then
        log_error "CA certificate not found. Run 'generate-ca' first."
        exit 1
    fi
    
    local client_key="$CERT_DIR/client-$client_name.key"
    local client_csr="$CERT_DIR/client-$client_name.csr"
    local client_cert="$CERT_DIR/client-$client_name.crt"
    local client_p12="$CERT_DIR/client-$client_name.p12"
    local ext_file="$CERT_DIR/client.ext"
    
    log_info "Generating client certificate for: $client_name"
    
    # Generate client private key
    openssl genrsa -out "$client_key" 2048
    chmod 600 "$client_key"
    
    # Generate CSR
    openssl req -new \
        -key "$client_key" \
        -out "$client_csr" \
        -subj "/CN=$client_name/O=Hafiz/OU=Client"
    
    # Create extensions file for client auth
    cat > "$ext_file" << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature
extendedKeyUsage = clientAuth
EOF
    
    # Sign with CA
    openssl x509 -req \
        -in "$client_csr" \
        -CA "$ca_cert" \
        -CAkey "$ca_key" \
        -CAcreateserial \
        -out "$client_cert" \
        -days "$DAYS_VALID" \
        -sha256 \
        -extfile "$ext_file"
    
    chmod 644 "$client_cert"
    
    # Also create PKCS#12 bundle for easy import
    openssl pkcs12 -export \
        -out "$client_p12" \
        -inkey "$client_key" \
        -in "$client_cert" \
        -certfile "$ca_cert" \
        -passout pass:
    
    rm -f "$client_csr" "$ext_file"
    
    log_info "Client certificate generated:"
    echo "  Certificate: $client_cert"
    echo "  Private Key: $client_key"
    echo "  PKCS#12: $client_p12 (empty password)"
    echo ""
    echo "To enable mTLS, set in config:"
    echo "  tls.require_client_cert = true"
    echo "  tls.client_ca_file = \"$ca_cert\""
}

# Verify certificate files
verify_certs() {
    check_openssl
    
    local cert_file="$CERT_DIR/server.crt"
    local key_file="$CERT_DIR/server.key"
    
    if [[ ! -f "$cert_file" ]]; then
        log_error "Certificate not found: $cert_file"
        exit 1
    fi
    
    if [[ ! -f "$key_file" ]]; then
        log_error "Key not found: $key_file"
        exit 1
    fi
    
    log_info "Verifying certificate and key..."
    
    # Verify certificate
    if openssl x509 -in "$cert_file" -noout 2>/dev/null; then
        log_info "Certificate: OK"
    else
        log_error "Certificate: INVALID"
        exit 1
    fi
    
    # Verify private key
    if openssl rsa -in "$key_file" -check -noout 2>/dev/null; then
        log_info "Private Key: OK"
    else
        log_error "Private Key: INVALID"
        exit 1
    fi
    
    # Verify key matches certificate
    cert_modulus=$(openssl x509 -noout -modulus -in "$cert_file" | md5sum)
    key_modulus=$(openssl rsa -noout -modulus -in "$key_file" | md5sum)
    
    if [[ "$cert_modulus" == "$key_modulus" ]]; then
        log_info "Key matches certificate: OK"
    else
        log_error "Key does NOT match certificate!"
        exit 1
    fi
    
    # Check expiration
    end_date=$(openssl x509 -enddate -noout -in "$cert_file" | cut -d= -f2)
    log_info "Certificate expires: $end_date"
    
    # Check if expired
    if openssl x509 -checkend 0 -noout -in "$cert_file" 2>/dev/null; then
        log_info "Certificate status: Valid"
    else
        log_warn "Certificate has EXPIRED!"
    fi
}

# Show certificate information
show_info() {
    check_openssl
    
    local cert_file="$CERT_DIR/server.crt"
    
    if [[ ! -f "$cert_file" ]]; then
        log_error "Certificate not found: $cert_file"
        exit 1
    fi
    
    log_info "Certificate Information:"
    echo ""
    openssl x509 -in "$cert_file" -noout -text | head -30
    echo ""
    log_info "Subject Alternative Names:"
    openssl x509 -in "$cert_file" -noout -ext subjectAltName 2>/dev/null || echo "  (none)"
}

# Parse command line arguments
COMMAND=""
DOMAIN="localhost"
CLIENT_NAME="client"

while [[ $# -gt 0 ]]; do
    case $1 in
        generate-self-signed|generate-ca|generate-server|generate-client|verify|info)
            COMMAND="$1"
            shift
            ;;
        --cert-dir)
            CERT_DIR="$2"
            shift 2
            ;;
        --days)
            DAYS_VALID="$2"
            shift 2
            ;;
        --domain)
            DOMAIN="$2"
            shift 2
            ;;
        --client-name)
            CLIENT_NAME="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Execute command
case $COMMAND in
    generate-self-signed)
        generate_self_signed "$DOMAIN"
        ;;
    generate-ca)
        generate_ca
        ;;
    generate-server)
        generate_server "$DOMAIN"
        ;;
    generate-client)
        generate_client "$CLIENT_NAME"
        ;;
    verify)
        verify_certs
        ;;
    info)
        show_info
        ;;
    "")
        usage
        exit 1
        ;;
esac

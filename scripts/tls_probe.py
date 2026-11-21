import socket, ssl, re, sys

HOST = "89.35.130.191"
PORTS = [22005, 22015, 22006, 22007, 443, 80]
socket.setdefaulttimeout(5)

for port in PORTS:
    print(f"\n=== {HOST}:{port} ===")
    # 1) Plain TCP reachability
    try:
        s = socket.create_connection((HOST, port), timeout=5)
        s.close()
        print("  TCP: open")
    except Exception as e:
        print(f"  TCP: {e}")
        continue
    # 2) TLS handshake, grab cert (ignore verification)
    try:
        ctx = ssl.create_default_context()
        ctx.check_hostname = False
        ctx.verify_mode = ssl.CERT_NONE
        with socket.create_connection((HOST, port), timeout=5) as sock:
            with ctx.wrap_socket(sock, server_hostname=HOST) as ss:
                der = ss.getpeercert(binary_form=True)
                print(f"  TLS: OK, version={ss.version()}, cert {len(der)} bytes")
                strs = [x.decode("ascii", "replace") for x in re.findall(rb"[ -~]{4,}", der)]
                # cert strings of interest: domains / CN / SAN / issuer
                interesting = [x for x in strs if any(k in x.lower() for k in (".com", ".fun", ".net", ".org", ".ru", "astra", "gta5rp", "grandrp", "CN", "Let's", "R3", "R10", "R11"))]
                print("  cert strings:", interesting[:30])
    except Exception as e:
        print(f"  TLS: {type(e).__name__}: {e}")

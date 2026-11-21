import socket, sys

HOST = "89.35.130.191"
PORTS = [22005, 22006, 22007]
PATHS = ["/info.json", "/players.json", "/", "/status"]

for port in PORTS:
    for path in PATHS:
        try:
            s = socket.create_connection((HOST, port), timeout=4)
            req = f"GET {path} HTTP/1.0\r\nHost: {HOST}\r\nConnection: close\r\n\r\n"
            s.sendall(req.encode())
            s.settimeout(4)
            chunks = []
            while True:
                try:
                    b = s.recv(4096)
                except socket.timeout:
                    break
                if not b:
                    break
                chunks.append(b)
                if sum(len(c) for c in chunks) > 8192:
                    break
            s.close()
            data = b"".join(chunks)
            head = data.split(b"\r\n\r\n", 1)
            status = head[0].split(b"\r\n")[0] if head else b""
            body = head[1] if len(head) > 1 else b""
            print(f"\n=== {HOST}:{port}{path} ===")
            print("  status:", status[:100])
            print("  body:", body[:600])
        except Exception as e:
            print(f"\n=== {HOST}:{port}{path} ===\n  {type(e).__name__}: {e}")

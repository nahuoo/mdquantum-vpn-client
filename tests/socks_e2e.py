"""Run after cargo build --locked --all-targets. Local processes only."""
import pathlib
import socket
import subprocess
import threading
import time

processes = []
cert = None

def exact(s, n):
    data = b''
    while len(data) < n:
        chunk = s.recv(n-len(data))
        assert chunk, 'unexpected EOF'
        data += chunk
    return data

def session():
    s = socket.create_connection(('127.0.0.1', 11080), timeout=5)
    s.sendall(b'\x05\x01\x00')
    assert exact(s, 2) == b'\x05\x00'
    return s

try:
    server = subprocess.Popen(['target/debug/examples/socks-server'], stdout=subprocess.PIPE, text=True)
    processes.append(server)
    assert '127.0.0.1' in server.stdout.readline()
    cert = server.stdout.readline().strip().split('Certificate: ')[1]
    client = subprocess.Popen(['target/debug/vpn-client', '--port', '11080', '--server-domain', 'localhost', '--server-port', '8443', '--server-cert', cert], stdout=subprocess.DEVNULL)
    processes.append(client)
    for _ in range(100):
        try:
            probe = socket.create_connection(('127.0.0.1',11080), timeout=.1)
            probe.close()
            break
        except OSError:
            time.sleep(.05)
    listener = socket.socket()
    listener.bind(('127.0.0.1',0)); listener.listen(); listener.settimeout(10)
    port = listener.getsockname()[1]
    def echo():
        with listener:
            conn, _ = listener.accept()
            with conn:
                conn.settimeout(5)
                data = exact(conn, 4)
                conn.sendall(data)
    thread = threading.Thread(target=echo)
    thread.start()
    with session() as s:
        # Domain is resolved by the relay, not by the SOCKS client.
        s.sendall(b'\x05\x01\x00\x03\x09localhost'+port.to_bytes(2,'big'))
        assert exact(s,10)[1] == 0
        s.sendall(b'ping'); assert exact(s,4) == b'ping'
    thread.join(timeout=10); assert not thread.is_alive()
    with session() as s:
        s.sendall(b'\x05\x03\x00\x01'); assert exact(s,10)[1] == 7
    with session() as s:
        s.sendall(b'\x05\x01\x00\x01\xc0\x00\x02\x01\x00\x50'); assert exact(s,10)[1] == 2
    with socket.create_connection(('127.0.0.1',11080),timeout=5) as s:
        s.sendall(b'\x05\x01\x02'); assert exact(s,2) == b'\x05\xff'
    print('PASS: TLS/SOCKS5 echo, server-side DNS, unsupported command, destination policy, unsupported authentication')
finally:
    for p in reversed(processes):
        p.terminate(); p.wait(timeout=5)
    if cert: pathlib.Path(cert).unlink(missing_ok=True)

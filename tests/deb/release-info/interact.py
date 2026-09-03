#!/usr/bin/env python3

import errno
import fcntl
import os
import pty
import select
import signal
import struct
import sys
import termios


def main() -> int:
    output_path, response, *command = sys.argv[1:]
    pid, fd = pty.fork()

    if pid == 0:
        os.environ["TERM"] = "xterm-256color"
        fcntl.ioctl(sys.stdout.fileno(), termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
        os.execvp(command[0], command)

    output = bytearray()
    replied = False

    while True:
        readable, _, _ = select.select([fd], [], [], 30)
        if not readable:
            os.kill(pid, signal.SIGTERM)
            sys.stderr.buffer.write(output)
            raise TimeoutError("timed out waiting for Nala")

        try:
            chunk = os.read(fd, 4096)
        except OSError as error:
            if error.errno == errno.EIO:
                break
            raise

        if not chunk:
            break

        output.extend(chunk)
        if b"\x1b[6n" in chunk:
            os.write(fd, b"\x1b[1;1R")
        if not replied and b"[y/N]" in output:
            os.write(fd, response.encode() + b"\r")
            replied = True

    _, status = os.waitpid(pid, 0)
    with open(output_path, "wb") as output_file:
        output_file.write(output)

    if not replied:
        sys.stderr.buffer.write(output)
        raise RuntimeError("Nala exited before prompting")

    return os.waitstatus_to_exitcode(status)


if __name__ == "__main__":
    sys.exit(main())

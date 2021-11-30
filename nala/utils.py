from logging.handlers import RotatingFileHandler, SysLogHandler
from os import geteuid, devnull
from pyshell import pyshell
from pathlib import Path
import logging

LICENSE = Path('/usr/share/common-licenses/GPL-3')
"""/usr/share/common-licenses/GPL-3"""
NALA_SOURCES = Path('/etc/apt/sources.list.d/nala-sources.list')
"""/etc/apt/sources.list.d/nala-sources.list"""
NALA_LOGDIR = Path('/var/log/nala')
"""/var/log/nala"""
NALA_LOGFILE = NALA_LOGDIR / 'nala.log'
"""/var/log/nala/nala.log"""

shell = pyshell(capture_output=True, text=True, check=True)

# Define log levels for import
INFO = 20
DEBUG = 10

# Create our main logger. This will do nothing unless we're root
logger = logging.getLogger('nala_logger')
logger.setLevel(INFO)

# Define logging formatters
formatter = logging.Formatter('%(asctime)s %(levelname)s: %(message)s', datefmt='[%Y-%m-%d %H:%M:%S]')
sys_formatter = logging.Formatter('nala: %(levelname)s: %(message)s')
nodate_format = logging.Formatter('[%(levelname)s]: %(message)s')
empty_format = logging.Formatter('%(message)s')

# Our syslogger. Currently only used for telling on people for using trying to use nala without permission
syslogger = logging.getLogger('nala_syslogger')
syslogger.setLevel(INFO)
syslog_handler = SysLogHandler(facility=SysLogHandler.LOG_USER, address='/dev/log')
syslog_handler.setFormatter(sys_formatter)
syslogger.addHandler(syslog_handler)

if geteuid() == 0:
	if not NALA_LOGDIR.exists():
		NALA_LOGDIR.mkdir()
	file_handler = RotatingFileHandler(NALA_LOGFILE, maxBytes=1024*1024, backupCount=10)
	file_handler.setFormatter(formatter)
	logger.addHandler(file_handler)
else:
	file_handler = RotatingFileHandler(devnull, maxBytes=1024*1024, backupCount=10)

syslog = syslogger.info
esyslog = syslogger.error

eprint = logger.error
iprint = logger.info
dprint = logger.debug

def logger_newline():
	file_handler.setFormatter(empty_format)
	iprint('')
	file_handler.setFormatter(formatter)

def ask(question):
	"""resp = input(f'{question}? [Y/n]

	Y returns True
	N returns False
	"""
	while True:
		resp = input(f'{question}? [Y/n] ')
		if resp in ['y', 'Y']:
			return True
		elif resp in ['n', 'N']:
			return False
		elif resp == '':
			return True
		else:
			print("Not a valid choice kiddo")

LION_ASCII = (
r"""
         |\_
       -' | \
      /7     \
     /        `-_
     \-'_        `-.____________
      -- \                 /    `.
         /                 \      \
 _______/    /_       ______\      |__________-
(,__________/  `-.___(,_____________----------_)
"""
)
# I couldn't find an artist for these. If anyone knows let me know.
# I love to give credit when I can
LION_ASCII2 = (
r"""
    |\_
  -' | `.
 /7      `-._
/            `-.____________
\-'_                        `-._
 -- `-._                    |` -`.
       |\               \   |   `\\
       | \  \______...---\_  \    \\
       |  \  \           | \  |    ``-.__--.
       |  |\  \         / / | |       ``---'
     _/  /_/  /      __/ / _| |
    (,__/(,__/      (,__/ (,__/
"""
)

CAT_ASCII = (
r"""
   |\---/|
   | ,_, |
    \_`_/-..----.
 ___/ `   ' ,""+ \  sk
(__...'   __\    |`.___.';
  (_,...'(_,.`__)/'.....+
"""
)
import logging
from logging.handlers import RotatingFileHandler, SysLogHandler
from os import devnull, geteuid

from nala.options import arguments
from nala.utils import NALA_DEBUGLOG, NALA_LOGDIR, NALA_LOGFILE

# Define log levels for import
INFO = 20
DEBUG = 10

# Define logging formatters
formatter = logging.Formatter(
	"%(asctime)s %(levelname)s: %(message)s", datefmt="[%Y-%m-%d %H:%M:%S]"
)
sys_formatter = logging.Formatter('nala: %(levelname)s: %(message)s')
nodate_format = logging.Formatter('[%(levelname)s]: %(message)s')
empty_format = logging.Formatter('%(message)s')

# Create our main logger. This will do nothing unless we're root
logger = logging.getLogger('nala_logger')
logger.setLevel(INFO)

# Our syslogger.
# Currently only used for telling on people for using trying to use nala without permission
syslogger = logging.getLogger('nala_syslogger')
syslogger.setLevel(INFO)
syslog_handler = SysLogHandler(facility=SysLogHandler.LOG_USER, address='/dev/log')
syslog_handler.setFormatter(sys_formatter)
syslogger.addHandler(syslog_handler)

if geteuid() == 0:
	if not NALA_LOGDIR.exists():
		NALA_LOGDIR.mkdir()
	if arguments.debug:
		file_handler = RotatingFileHandler(NALA_DEBUGLOG, maxBytes=1024**2, backupCount=10)
	else:
		file_handler = RotatingFileHandler(
			NALA_LOGFILE, maxBytes=1024**2, backupCount=10)
	file_handler.setFormatter(formatter)
	logger.addHandler(file_handler)
else:
	file_handler = RotatingFileHandler(devnull, maxBytes=1024**2, backupCount=10)

syslog = syslogger.info
esyslog = syslogger.error

eprint = logger.error
iprint = logger.info
dprint = logger.debug

if arguments.debug:
	std_err_handler = logging.StreamHandler()
	std_err_handler.setFormatter(nodate_format)
	logger.addHandler(std_err_handler)
	logger.setLevel(DEBUG)

# This allows us to print a blank newline in nala.log
def logger_newline():
	file_handler.setFormatter(empty_format)
	iprint('')
	file_handler.setFormatter(formatter)

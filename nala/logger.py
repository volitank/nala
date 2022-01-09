#                 __
#    ____ _____  |  | _____
#   /    \\__  \ |  | \__  \
#  |   |  \/ __ \|  |__/ __ \_
#  |___|  (____  /____(____  /
#       \/     \/          \/
#
# Copyright (C) 2021, 2022 Blake Lee
#
# This file is part of nala
#
# nala is program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# nala is program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with nala.  If not, see <https://www.gnu.org/licenses/>.
"""Nala logging Module."""
from __future__ import annotations

import logging
from logging.handlers import RotatingFileHandler, SysLogHandler
from os import devnull, geteuid

from nala.constants import NALA_DEBUGLOG, NALA_LOGDIR, NALA_LOGFILE
from nala.options import arguments

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

def logger_newline() -> None:
	"""Print a blank newline in nala.log."""
	file_handler.setFormatter(empty_format)
	iprint('')
	file_handler.setFormatter(formatter)
